use crate::Config;
use actix_web::{get, web, App, HttpServer, Responder};

pub async fn run_server(cfg: Config) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cfg.clone()))
            .service(index_handler)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index_handler(cfg: web::Data<Config>) -> impl Responder {
    format!("{} is up!", cfg.app_name)
}
