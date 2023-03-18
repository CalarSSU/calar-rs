use crate::{Config, Request};
use actix_web::{get, web, App, HttpServer, Responder};
use serde::Deserialize;

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
    _cfg: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    params: web::Query<OptParams>,
) -> impl Responder {
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

    format!("{req:#?}")
}
