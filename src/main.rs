mod db;
mod embedding;
mod ino_api;
mod ino_checker;
mod ner;
mod rv;
mod utils;

use dotenv::dotenv;

use crate::ino_api::handlers;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};

use crate::ner::entities::PythonEntities;
use std::env;

use crate::ino_checker::new_checker::WarningNamesChecker;
use crate::ino_checker::new_name_checker::NameChecker;

use crate::embedding::vectorize::YandexEmbedding;

use crate::db::interface::DB;
use crate::db::sqlite::Database;

use crate::ino_api::server_api::Checker as api_checker;

use std::sync::Arc;
use tokio::sync::Mutex;

use std::io::Write;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    env_logger::init();

    let entities_url = env::var("ENTITIES_URL").ok().unwrap();
    let rv_entities = PythonEntities::new(entities_url);

    let name_checker = NameChecker::new();

    let model = env::var("YANDEX_MODEL").ok().unwrap();
    let token = env::var("YANDEX_SECRET").ok().unwrap();
    let url = env::var("YANDEX_URL").ok().unwrap();
    let yandex_embedding = YandexEmbedding::new(model, token, url);

    let db = Arc::new(Mutex::new(
        Database::new("assets/db/ino.sqlite").ok().unwrap(),
    ));
    let warning_names = db.lock().await.get_all().ok().unwrap();

    let warning_name_checker = Mutex::new(WarningNamesChecker::new(
        warning_names,
        yandex_embedding,
        name_checker,
        rv_entities,
    ));

    println!("Запуск сервера");
    std::io::stdout().flush().unwrap();

    let need_full_data = true;

    let api_checker = api_checker::new(need_full_data, warning_name_checker, db);
    let checker_data = web::Data::new(api_checker);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(
                "%a %t \"%r\" %s %b \"%{User-Agent}i\" %Dms \"%{Content-Type}i\"",
            ))
            .app_data(checker_data.clone())
            .route("/check/{id}", web::get().to(handlers::check_by_id_handler))
            .route("/check", web::post().to(handlers::check_by_text))
            .route("/update", web::get().to(handlers::update_inos))
            .route("/add", web::post().to(handlers::add_new_names))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
