use crate::ino_api::server_api::{Checker, ErrorS};
use actix_web::{HttpResponse, web};

pub async fn check_handler(checker: web::Data<Checker>, path: web::Path<String>) -> HttpResponse {
    checker.check(path.into_inner()).await
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
