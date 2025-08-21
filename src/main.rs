mod db;
mod embedding;
mod ino_api;
mod ino_checker;
mod ino_loader;
mod ner;
mod rv;

use dotenv::dotenv;

use crate::ino_api::{handlers, server_api};

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};

use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let mut need_full_data: bool = false;

    match env::var("FULL_DATA") {
        Err(_) => {}
        Ok(s) => need_full_data = s.to_lowercase() == "true",
    }

    // инициализация Checker
    let checker = server_api::Checker::new("assets/db/ino.sqlite", need_full_data)
        .ok()
        .unwrap();

    // оборачиваем в web::Data для шаринга
    let checker_data = web::Data::new(checker);

    env_logger::init();

    use std::io::Write;
    println!("Запуск сервера");
    std::io::stdout().flush().unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(
                "%a %t \"%r\" %s %b \"%{User-Agent}i\" %Dms \"%{Content-Type}i\"",
            ))
            .app_data(checker_data.clone())
            .route("/check/{id}", web::get().to(handlers::check_by_id_handler))
            .route("/check", web::post().to(handlers::check_by_text))
            .route("/update", web::get().to(handlers::update_inos))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
