use std::time::Duration;

use crate::db::sqlite as my_sqlite;
use crate::ino_checker::checker::get_inos;
use actix_web::HttpResponse;
use anyhow::Error;
use serde::Serialize;
use tokio::time::sleep;

use std::sync::RwLock;

pub struct Checker {
    warning_names: RwLock<Vec<my_sqlite::Record>>,
    db_path: String,
}

#[derive(Serialize)]
pub struct ErrorS {
    pub error: String,
}

impl Checker {
    pub fn new(db_path: &str) -> Result<Self, Error> {
        let db = my_sqlite::Database::new(db_path)?;
        let warning_names = db.get_all()?;
        Ok(Checker {
            warning_names: RwLock::new(warning_names),
            db_path: db_path.to_string(),
        })
    }

    pub async fn check(&self, id: String) -> HttpResponse {
        let mut i: u8 = 0;
        loop {
            let names = { self.warning_names.read().unwrap().clone() };
            match get_inos(&id, names).await {
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

    pub fn update_warning_names(&self) -> Result<(), Error> {
        // TODO: реализовать само обновление элементов в базе

        let db = my_sqlite::Database::new(&self.db_path)?;
        let warning_names = db.get_all()?;
        *self.warning_names.write().unwrap() = warning_names;
        Ok(())
    }
}
