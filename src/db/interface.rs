use crate::db::model::Record;
use anyhow::Result;

pub trait DB {
    fn get_all(&self) -> Result<Vec<Record>>;
}
