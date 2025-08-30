use crate::db::model::Record;
use anda_db_hnsw::{DistanceMetric, HnswConfig, HnswIndex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ANNIndex {
    index: HnswIndex,
    records: Vec<Record>,
    dim: usize,
}

impl ANNIndex {
    /// name — любое имя индекса; dim — размерность embedding
    pub fn new(name: impl Into<String>, dim: usize) -> Self {
        let config = HnswConfig {
            dimension: dim,
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };
        // при желании: config.ef_search = 64; config.ef_construction = 200; и т.д.

        let index = HnswIndex::new(name.into(), Some(config));
        Self {
            index,
            records: Vec::new(),
            dim,
        }
    }

    /// Добавление записи. ID в индексе = позиция в векторе `records`
    pub fn add(&mut self, rec: Record) {
        assert!(
            rec.embedding.len() == self.dim,
            "Embedding len {} != dim {}",
            rec.embedding.len(),
            self.dim
        );
        let id = self.records.len() as u64;
        self.index
            .insert_f32(id, rec.embedding.clone(), now_ms())
            .expect("insert_f32 failed");
        self.records.push(rec);
    }

    /// Поиск k ближайших. Возвращает только не удалённые записи.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<Record> {
        assert!(
            query.len() == self.dim,
            "Query len {} != dim {}",
            query.len(),
            self.dim
        );
        let matches = self.index.search_f32(query, k).expect("search_f32 failed");

        // `search_f32` возвращает (id, distance), отсортировано по возрастанию distance
        matches
            .into_iter()
            .filter_map(|(id, _dist)| {
                self.records
                    .get(id as usize)
                    .filter(|r| !r.is_removed)
                    .cloned()
            })
            .collect()
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_millis() as u64
}
