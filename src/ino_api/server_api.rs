use crate::ino_checker::checker::get_inos;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorS {
    error: String,
}

#[get("/check/{id}")]
pub async fn check(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    let db_path = "assets/db/ino.sqlite";
    let inos = get_inos(&id, db_path).await;
    match inos {
        Err(e) => {
            eprintln!("{e}");
            let err = ErrorS {
                error: format!("{e}"),
            };
            HttpResponse::InternalServerError().json(err)
        }
        Ok(inos) => HttpResponse::Ok().json(inos),
    }
}
