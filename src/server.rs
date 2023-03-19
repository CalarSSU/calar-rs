use crate::{config, tracto, validate_request, Config, Request};
use icalendar::Calendar;

use actix_web::{get, web};
use serde::Deserialize;
use std::{io::Write, path::PathBuf};

#[derive(Debug, thiserror::Error)]
enum ServerError {
    #[error("Internal error: {msg}")]
    InternalError { msg: String },

    #[error("Bad request: {msg}")]
    BadRequest { msg: String },
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
        Self::InternalError { msg: e.to_string() }
    }
}

#[derive(Debug, Deserialize)]
struct OptParams {
    subgroups: Option<String>,
    translator: Option<bool>,
}

pub async fn run_server(cfg: Config, prune: bool) -> std::io::Result<()> {
    if prune {
        std::fs::remove_dir_all(get_cache_dir())?;
    }

    let (addr, port) = (cfg.addr.clone(), cfg.port);

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(web::Data::new(cfg.clone()))
            .service(index_handler)
            .service(request_cal_handler)
    })
    .bind((addr, port))?
    .run()
    .await
}

#[get("/")]
async fn index_handler(cfg: web::Data<Config>) -> String {
    format!("{} is up!", cfg.app_name)
}

#[get("/{department}/{form}/{group}")]
async fn request_cal_handler(
    cfg: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    params: web::Query<OptParams>,
) -> Result<actix_files::NamedFile, ServerError> {
    let (department, form, group) = path.into_inner();
    let translator = params.translator.unwrap_or(false);
    let subgroups: Vec<String> = match params.subgroups.clone() {
        None => Vec::new(),
        Some(s) => serde_json::from_str(s.as_str()).unwrap(),
    };
    let req = Request {
        department,
        form,
        group,
        translator,
        subgroups,
    };

    if let Err(e) = validate_request(&cfg, &req).await {
        return Err(ServerError::BadRequest { msg: e.to_string() });
    };

    let file_path = match look_up_in_cache(&req) {
        Some(file_path) => file_path,
        None => {
            let schedule = tracto::fetch_schedule(&cfg, &req)
                .await
                .map_err(|e| ServerError::InternalError { msg: e.to_string() })?;
            let calendar = schedule.to_ical(&cfg, &req);
            save_to_cache(&req, calendar)?
        }
    };

    Ok(actix_files::NamedFile::open(file_path)?)
}

fn gen_filename(req: &Request) -> String {
    format!(
        "{}-{}-{}-{}{}.ics",
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

    proj_dirs.cache_dir().to_owned()
}

fn save_to_cache(req: &Request, calendar: Calendar) -> Result<PathBuf, ServerError> {
    let cache_dir = get_cache_dir();
    std::fs::create_dir_all(cache_dir.clone())?;
    let file_path = cache_dir.join(gen_filename(req));

    let mut file = std::fs::File::create(file_path.clone())?;
    file.write_all(calendar.to_string().as_bytes())?;

    Ok(file_path)
}

fn look_up_in_cache(req: &Request) -> Option<PathBuf> {
    let file_path = get_cache_dir().join(gen_filename(req));

    if file_path.exists() {
        Some(file_path)
    } else {
        None
    }
}
