use crate::db::model::Record;
use anyhow::Result;

pub trait DB {
    fn insert(&self, record: &Record) -> Result<()>;
    fn update_vector(&self, id: i32, vec: &[f32]) -> Result<()>;
    fn get_all(&self) -> Result<Vec<Record>>;
}
