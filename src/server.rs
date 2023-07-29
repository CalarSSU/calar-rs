use crate::{
    cache, config,
    models::{ExamList, Schedule},
    tracto::{self, find_subgroups, validate_request},
    Config, Request,
};

use actix_web::{get, middleware::Logger, web};
use serde::Deserialize;
use std::process::ExitCode;

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
            .service(
                web::scope(&format!("/v{}", config::VERSION))
                    .service(index_handler)
                    .service(subgroups_handler)
                    .service(schedule_handler)
                    .service(exams_handler),
            )
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

#[get("/schedule/{department}/{form}/{group}")]
async fn schedule_handler(
    cfg: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    params: web::Query<OptParams>,
) -> Result<actix_files::NamedFile, ServerError> {
    let (department, form, group) = path.into_inner();
    let translator = params.translator.unwrap_or(false);
    let subgroups = match &params.subgroups {
        Some(s) => {
            serde_json::from_str(s.as_str()).map_err(|e| ServerError::BadRequest(e.to_string()))?
        }
        None => Vec::new(),
    };
    let req = Request {
        department,
        form,
        group,
        translator,
        subgroups,
    };

    validate_request(&cfg, &req)
        .await
        .map_err(|e| ServerError::BadRequest(e.to_string()))?;

    let file_path = match cache::look_up_in_cache::<Schedule>(&req) {
        Some(file_path) => file_path,
        None => {
            let schedule = tracto::fetch_schedule(&cfg, &req)
                .await
                .map_err(|e| ServerError::InternalError(e.to_string()))?;
            let calendar = schedule.to_ical(&cfg, &req);
            cache::save_to_cache::<Schedule>(&req, calendar)
                .map_err(|e| ServerError::InternalError(e.to_string()))?
        }
    };

    actix_files::NamedFile::open(file_path).map_err(|e| e.into())
}

#[get("/exams/{department}/full/{group}")]
async fn exams_handler(
    cfg: web::Data<Config>,
    path: web::Path<(String, String)>,
) -> Result<actix_files::NamedFile, ServerError> {
    let (department, group) = path.into_inner();
    let req = Request {
        department,
        form: "full".to_string(),
        group,
        translator: false,
        subgroups: Vec::new(),
    };

    validate_request(&cfg, &req)
        .await
        .map_err(|e| ServerError::BadRequest(e.to_string()))?;

    let file_path = match cache::look_up_in_cache::<ExamList>(&req) {
        Some(file_path) => file_path,
        None => {
            let schedule = tracto::fetch_exam(&cfg, &req)
                .await
                .map_err(|e| ServerError::InternalError(e.to_string()))?;
            let calendar = schedule.to_ical();
            cache::save_to_cache::<ExamList>(&req, calendar)
                .map_err(|e| ServerError::InternalError(e.to_string()))?
        }
    };

    actix_files::NamedFile::open(file_path).map_err(|e| e.into())
}
