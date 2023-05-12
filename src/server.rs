use crate::{
    config,
    tracto::{self, find_subgroups, validate_request},
    Config, Request,
    models::{Schedule, ExamList},
};
use icalendar::Calendar;

use actix_web::{get, middleware::Logger, web};
use serde::Deserialize;
use std::{io::Write, path::PathBuf, process::ExitCode};

#[derive(Debug, thiserror::Error)]
enum ServerError {
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl actix_web::error::ResponseError for ServerError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code())
            .insert_header(actix_web::http::header::ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            ServerError::InternalError { .. } => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::BadRequest { .. } => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(e: std::io::Error) -> Self {
        Self::InternalError(e.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct OptParams {
    subgroups: Option<String>,
    translator: Option<bool>,
}

pub async fn run_server(cfg: Config) -> ExitCode {
    let (addr, port) = (cfg.addr.clone(), cfg.port);

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(Logger::new("%{r}a %r %s | %T sec."))
            .app_data(web::Data::new(cfg.clone()))
            .service(index_handler)
            .service(subgroups_handler)
            .service(request_cal_handler)
            .service(request_exam_handler)
            .service(another_request)
    })
    .bind((addr, port));

    if let Err(e) = server {
        eprintln!("Cannot start server: {e}");
        return ExitCode::FAILURE;
    }
    let server = server.unwrap();

    if let Err(e) = server.run().await {
        eprintln!("Cannot start server: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

#[get("/")]
async fn index_handler(cfg: web::Data<Config>) -> String {
    format!("{} is up!", cfg.app_name)
}

#[get("/subgroups/{department}/{form}/{group}")]
async fn subgroups_handler(
    cfg: web::Data<Config>,
    path: web::Path<(String, String, String)>,
) -> Result<String, ServerError> {
    let (department, form, group) = path.into_inner();
    let req = Request {
        department,
        form,
        group,
        translator: false,
        subgroups: Vec::new(),
    };

    let schedule = tracto::fetch_schedule(&cfg, &req)
        .await
        .map_err(|e| ServerError::InternalError(e.to_string()))?;

    let subgroups = find_subgroups(&schedule);

    Ok(serde_json::to_string(&subgroups).unwrap_or("[]".to_string()))
}

#[get("/{department}/{form}/{group}")]
async fn request_cal_handler(
    cfg: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    params: web::Query<OptParams>,
) -> Result<actix_files::NamedFile, ServerError> {
    let (department, form, group) = path.into_inner();
    let translator = params.translator.unwrap_or(false);
    let subgroups: Vec<String> = match &params.subgroups {
        None => Vec::new(),
        Some(s) => match serde_json::from_str(s.as_str()) {
            Ok(v) => v,
            Err(e) => return Err(
                ServerError::BadRequest(format!("Cannot parse subroups: {}", e.to_string()))
            )
        },
    };
    let req = Request {
        department,
        form,
        group,
        translator,
        subgroups,
    };

    if let Err(e) = validate_request(&cfg, &req).await {
        return Err(ServerError::BadRequest(e.to_string()));
    };

    let file_path = match look_up_in_cache::<Schedule>(&req) {
        Some(file_path) => file_path,
        None => {
            let schedule = tracto::fetch_schedule(&cfg, &req)
                .await
                .map_err(|e| ServerError::InternalError(e.to_string()))?;
            let calendar = schedule.to_ical(&cfg, &req);
            save_to_cache::<Schedule>(&req, calendar)?
        }
    };

    Ok(actix_files::NamedFile::open(file_path)?)
}

#[get("/exam/{department}/full/{group}")]
async fn request_exam_handler(
    cfg: web::Data<Config>,
    path: web::Path<(String, String)>,
) -> Result<actix_files::NamedFile, ServerError> {
    let (department, group) = path.into_inner();
    let form = "full";
    let translator = false;
    let subgroups = Vec::new();
    let req = Request {
        department,
        form: form.to_string(),
        group,
        translator,
        subgroups,
    };

    if let Err(e) = validate_request(&cfg, &req).await {
        return Err(ServerError::BadRequest(e.to_string()));
    };

    let file_path = match look_up_in_cache::<ExamList>(&req) {
        Some(file_path) => file_path,
        None => {
            let schedule = tracto::fetch_exam(&cfg, &req)
                .await
                .map_err(|e| ServerError::InternalError(e.to_string()))?;
            let calendar = schedule.to_ical();
            save_to_cache::<ExamList>(&req, calendar)?
        }
    };

    Ok(actix_files::NamedFile::open(file_path)?)
}

#[get("/{tail:.*}")]
async fn another_request(path: web::Path<String>) -> String {
    let tail = path.into_inner();
    log::error!("Another request {}", tail);
    format!("Aboba")
}

pub fn gen_filename<T>(req: &Request) -> String {
    let tmp_vec: Vec<&str> = std::any::type_name::<T>().split("::").collect();
    format!(
        "{}-{}-{}-{}-{}{}.ics",
        tmp_vec[tmp_vec.len() - 1],
        req.department,
        req.form,
        req.group,
        req.subgroups.join("_"),
        if req.translator { "-t" } else { "" }
    )
}

fn get_cache_dir() -> PathBuf {
    let proj_dirs =
        directories::ProjectDirs::from(config::QUALIFIER, config::ORG_NAME, config::APP_NAME)
            .expect("No valid config directory could be retrieved from the operating system");

    proj_dirs.cache_dir().join("calendars")
}

fn save_to_cache<T>(req: &Request, calendar: Calendar) -> Result<PathBuf, ServerError> {
    let cache_dir = get_cache_dir();
    std::fs::create_dir_all(cache_dir.clone())?;
    let file_path = cache_dir.join(gen_filename::<T>(req));

    let mut file = std::fs::File::create(file_path.clone())?;
    file.write_all(calendar.to_string().as_bytes())?;

    Ok(file_path)
}

fn look_up_in_cache<T>(req: &Request) -> Option<PathBuf> {
    let file_path = get_cache_dir().join(gen_filename::<T>(req));

    if file_path.exists() {
        Some(file_path)
    } else {
        None
    }
}

pub fn prune_cache() -> ExitCode {
    match std::fs::remove_dir_all(get_cache_dir()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
