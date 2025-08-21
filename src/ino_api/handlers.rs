use crate::ino_api::server_api::{Checker, ErrorS};
use crate::rv::get::strip_html;
use actix_web::{HttpResponse, web};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextRequest {
    pub text: String,
}

pub async fn check_by_text(
    checker: web::Data<Checker>,
    req: web::Json<TextRequest>,
) -> HttpResponse {
    let text = &req.text;
    checker
        .check_by_text(strip_html(text.clone()), checker.need_full_data)
        .await
}

pub async fn check_by_id_handler(
    checker: web::Data<Checker>,
    path: web::Path<String>,
) -> HttpResponse {
    checker
        .check_by_id(path.into_inner(), checker.need_full_data)
        .await
}

pub async fn update_inos(checker: web::Data<Checker>) -> HttpResponse {
    match checker.update_warning_names() {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            let err: ErrorS = ErrorS {
                error: format!("{e}"),
            };
            HttpResponse::InternalServerError().json(err)
        }
    }
}
