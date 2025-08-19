use anyhow::Error;
use serde::Serialize;
use std::env;

use crate::db::sqlite as my_sqlite;
use crate::embedding::vectorize::get_embedding;
use crate::ner::entities::get_entities;
use crate::rv::get::getText;

use crate::ino_checker::utils::{get_must_relevant, unordered_levenshtein};

#[derive(Debug, Serialize)]
pub struct Doc {
    pub status: String,
    pub name: String,
    pub is_removed: bool,
}

#[derive(Debug, Serialize)]
pub struct WarningName {
    pub name: String,
    pub norm_name: String,
    pub name_type: String,
    pub docs: Vec<Doc>,
}

#[derive(Debug, Serialize)]
pub struct WarningNames {
    pub warnings: Vec<WarningName>,
    pub accepted_names: Vec<String>,
}

pub async fn get_inos(news_id: &str, db_path: &str) -> Result<WarningNames, Error> {
    let yandex_token: String = env::var("YANDEX_SECRET").ok().unwrap();
    let yandex_model: String = env::var("YANDEX_MODEL").ok().unwrap();
    let yandex_url: String = env::var("YANDEX_URL").ok().unwrap();
    let embeddibg_url: String = env::var("ENTITIES_URL").ok().unwrap();

    let mut inos: Vec<WarningName> = Vec::new();
    let mut accepted_names: Vec<String> = Vec::new();

    let db = my_sqlite::Database::new(db_path)?;
    let inoagents = db.get_all()?;
    let text = getText(news_id).await?;

    let entities = get_entities(&text, &embeddibg_url).await?.entities;

    for entity in entities.iter() {
        let name = entity.name.clone();
        let entity_type = entity.entity_type.clone();
        let embedding = get_embedding(
            &entity.name.to_lowercase(),
            &yandex_model,
            &yandex_token,
            &yandex_url,
        )
        .await?
        .embedding
        .unwrap();

        let most_relevant = get_must_relevant(&embedding, &inoagents.clone(), 5, 0.75);

        let mut ino = WarningName {
            name: name.clone(),
            norm_name: entity.norm_name.clone(),
            name_type: entity_type.clone(),
            docs: Vec::new(),
        };

        for ag in most_relevant {
            let dis = unordered_levenshtein(
                &entity.norm_name.to_lowercase().clone(),
                &ag.name.to_lowercase().clone(),
            );
            if dis <= 20 {
                let doc = Doc {
                    name: ag.name.clone(),
                    is_removed: ag.is_removed,
                    status: ag.record_type,
                };
                ino.docs.push(doc);
            }
        }

        if !ino.docs.is_empty() {
            inos.push(ino);
        } else {
            accepted_names.push(ino.name);
        }
    }

    let res = WarningNames {
        warnings: inos,
        accepted_names,
    };
    Ok(res)
}
