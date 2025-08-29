use crate::db::model::Record;
use anyhow::Result;

pub trait DB {
    /// Получение всех записей из базы данных
    fn get_all(&self) -> Result<Vec<Record>>;
}
