use crate::db::interface::DB;
use crate::db::model::Record;
use rusqlite::{Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                embedding BLOB NOT NULL,
                is_removed INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        Ok(Self { conn })
    }
}

impl DB for Database {
    /// Чтение всех записей
    fn get_all(&self) -> Result<Vec<Record>, anyhow::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, type, embedding, is_removed FROM records")?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let record_type: String = row.get(1)?;
            let blob: Vec<u8> = row.get(2)?;
            let is_removed: i32 = row.get(3)?;

            let embedding = blob
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();

            Ok(Record {
                name,
                record_type,
                embedding,
                is_removed: is_removed != 0,
            })
        })?;

        let mut records = Vec::new();
        for rec in rows {
            records.push(rec?);
        }
        Ok(records)
    }
}
