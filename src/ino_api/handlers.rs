use crate::ino_api::server_api::{Checker, ErrorS};
use actix_web::{HttpResponse, web};
use serde::Deserialize;

use crate::ino_checker::interface::BasicChecker;

use crate::db::sqlite::Database;
use crate::embedding::vectorize::YandexEmbedding;
use crate::ino_checker::new_checker::WarningNamesChecker;
use crate::ino_checker::new_name_checker::NameChecker;
use crate::ner::entities::PythonEntities;

pub type ApiChecker =
    Checker<WarningNamesChecker<YandexEmbedding, NameChecker, PythonEntities>, Database>;

#[derive(Deserialize)]
pub struct TextRequest {
    pub text: String,
}

pub async fn check_by_text(
    checker: web::Data<ApiChecker>,
    req: web::Json<TextRequest>,
) -> HttpResponse {
    let result = checker
        .checker
        .lock()
        .unwrap()
        .get_inos_from_text(&req.text, checker.need_full_data)
        .await;

    match result {
        Ok(inos) => HttpResponse::Ok().json(inos),
        Err(e) => HttpResponse::InternalServerError().json(ErrorS {
            error: format!("{e}"),
        }),
    }
}

pub async fn check_by_id_handler(
    checker: web::Data<ApiChecker>,
    path: web::Path<String>,
) -> HttpResponse {
    let id = path.into_inner();
    let result = checker
        .checker
        .lock()
        .unwrap()
        .get_inos(&id, checker.need_full_data)
        .await;

    match result {
        Ok(inos) => HttpResponse::Ok().json(inos),
        Err(e) => HttpResponse::InternalServerError().json(ErrorS {
            error: format!("{e}"),
        }),
    }
}

pub async fn update_inos(checker: web::Data<ApiChecker>) -> HttpResponse {
    // блокируем доступ к изменяемым полям внутри Checker
    let result = checker.update_warning_names();

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ErrorS {
            error: format!("{e}"),
        }),
    }
}
