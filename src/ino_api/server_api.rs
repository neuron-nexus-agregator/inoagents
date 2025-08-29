use crate::db::interface::DB;
use std::time::Duration;

use crate::ino_checker::interface::BasicChecker;
use actix_web::HttpResponse;
use anyhow::Error;
use serde::Serialize;
use tokio::time::sleep;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::model::Record;

pub struct Checker<T: BasicChecker, D: DB> {
    pub need_full_data: bool,
    checker: Mutex<T>,
    database: Arc<Mutex<D>>,
}

#[derive(Serialize)]
pub struct ErrorS {
    pub error: String,
}

impl<T: BasicChecker, D: DB> Checker<T, D> {
    pub fn new(
        need_full_data: bool,
        checker: Mutex<T>,
        database: Arc<Mutex<D>>,
    ) -> Result<Self, Error> {
        Ok(Checker {
            need_full_data,
            checker,
            database,
        })
    }

    pub async fn check_by_id(&self, id: String, need_full_data: bool) -> HttpResponse {
        let mut i: u8 = 0;
        loop {
            match self
                .checker
                .lock()
                .await
                .get_inos(&id, need_full_data)
                .await
            {
                Err(e) => {
                    i += 1;

                    if i >= 4 {
                        let err = ErrorS {
                            error: format!("{e}"),
                        };
                        return HttpResponse::InternalServerError().json(err);
                    }

                    eprintln!("{e}");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                Ok(inos) => return HttpResponse::Ok().json(inos),
            }
        }
    }

    pub async fn check_by_text(&self, text: String, need_full_data: bool) -> HttpResponse {
        let mut i: u8 = 0;
        loop {
            match self
                .checker
                .lock()
                .await
                .get_inos_from_text(&text, need_full_data)
                .await
            {
                Err(e) => {
                    i += 1;

                    if i >= 4 {
                        let err = ErrorS {
                            error: format!("{e}"),
                        };
                        return HttpResponse::InternalServerError().json(err);
                    }

                    eprintln!("{e}");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                Ok(inos) => return HttpResponse::Ok().json(inos),
            }
        }
    }

    pub async fn update_warning_names(&self) -> Result<(), Error> {
        let new_warning_names = self.database.lock().await.get_all()?;
        self.checker
            .lock()
            .await
            .change_warning_names(new_warning_names);
        Ok(())
    }

    pub async fn add_warning_names(&self, names: Vec<Record>) -> HttpResponse {
        self.checker.lock().await.add_warning_names(names);
        HttpResponse::Ok().finish()
    }
}
