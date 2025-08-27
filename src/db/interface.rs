use crate::db::models::Record;

pub trait DB {
    fn insert(&self, record: &Record) -> Result<(), anyhow::Error>;
    fn get_all(&self) -> Result<Vec<Record>, anyhow::Error>;
}
