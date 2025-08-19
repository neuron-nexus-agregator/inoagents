mod db;
mod embedding;
mod ino_api;
mod ino_checker;
mod ino_loader;
mod ner;
mod rv;

use dotenv::dotenv;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

use crate::ino_checker::checker::get_inos;

use crate::db::sqlite;
use crate::embedding::vectorize::get_embedding;
use crate::ino_loader::loader::load;

use crate::ino_api::server_api;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    //load_inos().await;
    println!("Запуск сервера");
    HttpServer::new(|| App::new().service(server_api::check))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

async fn load_inos() {
    let yandex_token: String = env::var("YANDEX_SECRET").ok().unwrap();
    let yandex_model: String = env::var("YANDEX_MODEL").ok().unwrap();
    let yandex_url: String = env::var("YANDEX_URL").ok().unwrap();

    let inos = load("assets/nezh.xlsx", "nezh").ok().unwrap();
    //let inos = load("assets/export.xlsx", "ino").ok().unwrap();
    let db = sqlite::Database::new("assets/db/ino.sqlite").ok().unwrap();
    let mut i = 0;

    for ino in inos.iter() {
        i += 1;
        loop {
            sleep(Duration::from_millis(100)).await; // базовая задержка между запросами
            let embedding_res_row = get_embedding(
                &ino.name.clone().to_lowercase(),
                &yandex_model,
                &yandex_token,
                &yandex_url,
            )
            .await;

            match embedding_res_row {
                Err(e) => {
                    eprintln!("{i}/{} — ошибка: {e}, повтор через 1 секунду", inos.len());
                    sleep(Duration::from_secs(1)).await;
                    continue; // повторяем запрос
                }
                Ok(embedding_res) => {
                    if let Some(e) = embedding_res.error {
                        println!(
                            "Ошибка в ответе {i}/{}: {e}, повтор через 1 секунду",
                            inos.len()
                        );
                        sleep(Duration::from_secs(1)).await;
                        continue; // повторяем запрос
                    }

                    let embedding = embedding_res.embedding.unwrap();

                    let rec = sqlite::Record {
                        name: ino.name.clone(),
                        record_type: ino.status.clone(),
                        is_removed: ino.is_removed,
                        embedding,
                    };

                    if let Err(e) = db.insert(&rec) {
                        eprintln!("Ошибка при записи {i}/{}: {e}", inos.len());
                        sleep(Duration::from_secs(1)).await;
                        continue; // повторяем запись
                    } else {
                        println!("{i}/{} — успешно записан", inos.len());
                    }

                    break; // выходим из цикла, идем к следующему элементу
                }
            }
        }
    }
}

async fn check(id: &str) {
    let db_path = "assets/db/ino.sqlite";
    let inos = get_inos(id, db_path).await;
    match inos {
        Err(e) => eprintln!("{e}"),
        Ok(inos) => println!("{inos:?}"),
    }
}
