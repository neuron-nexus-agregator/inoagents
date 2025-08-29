use crate::ino_api::server_api::{Checker, ErrorS};
use actix_web::{HttpResponse, web};
use serde::Deserialize;

use crate::db::model::Record;
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

#[derive(Deserialize, Clone)]
pub struct Records {
    pub records: Vec<Record>,
}

pub async fn add_new_names(
    checker: web::Data<ApiChecker>,
    req: web::Json<Records>,
) -> HttpResponse {
    checker.add_warning_names(req.records.clone()).await
}

pub async fn check_by_text(
    checker: web::Data<ApiChecker>,
    req: web::Json<TextRequest>,
) -> HttpResponse {
    checker
        .check_by_text(req.text.clone(), checker.need_full_data)
        .await
}

pub async fn check_by_id_handler(
    checker: web::Data<ApiChecker>,
    path: web::Path<String>,
) -> HttpResponse {
    checker
        .check_by_id(path.into_inner(), checker.need_full_data)
        .await
}

pub async fn update_inos(checker: web::Data<ApiChecker>) -> HttpResponse {
    // блокируем доступ к изменяемым полям внутри Checker
    let result = checker.update_warning_names().await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ErrorS {
            error: format!("{e}"),
        }),
    }
}
