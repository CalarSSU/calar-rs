use crate::{config, tracto, validate_request, Config, Request};
use actix_files::NamedFile;
use actix_web::{
    error, get,
    http::{header::ContentType, StatusCode},
    web, App, HttpResponse, HttpServer, Responder,
};
use icalendar::Calendar;
use serde::Deserialize;
use std::{fs::File, io::Write, path::PathBuf};

#[derive(Debug, thiserror::Error)]
enum ServerError {
    #[error("Internal request: {msg}")]
    InternalError { msg: String },

    #[error("Bad request: {msg}")]
    BadRequest { msg: String },
}

impl error::ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ServerError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::BadRequest { .. } => StatusCode::BAD_REQUEST,
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

pub async fn run_server(cfg: Config) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cfg.clone()))
            .service(index_handler)
            .service(request_cal_handler)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index_handler(cfg: web::Data<Config>) -> impl Responder {
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

    let schedule = tracto::fetch_schedule(&cfg, &req)
        .await
        .map_err(|e| ServerError::InternalError { msg: e.to_string() })?;
    let calendar = schedule.to_ical(&cfg, &req);
    let file_path = save_to_cache(&req, calendar)?;

    Ok(NamedFile::open(file_path)?)
}

fn save_to_cache(req: &Request, calendar: Calendar) -> Result<PathBuf, ServerError> {
    let proj_dirs =
        directories::ProjectDirs::from(config::QUALIFIER, config::ORG_NAME, config::APP_NAME)
            .expect("No valid config directory could be retrieved from the operating system");
    let filename = format!(
        "{}-{}-{}-{}{}.ics",
        req.department,
        req.form,
        req.group,
        req.subgroups.join("_"),
        if req.translator { "-t" } else { "" }
    );
    let file_path = proj_dirs.cache_dir().join(filename);

    let mut file = File::create(file_path.clone())?;
    file.write_all(calendar.to_string().as_bytes())?;

    Ok(file_path)
}
