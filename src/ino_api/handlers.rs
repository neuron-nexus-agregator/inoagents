use crate::ino_api::server_api::Checker;
use actix_web::{HttpResponse, web};

pub async fn check_handler(checker: web::Data<Checker>, path: web::Path<String>) -> HttpResponse {
    checker.check(path.into_inner()).await
}
