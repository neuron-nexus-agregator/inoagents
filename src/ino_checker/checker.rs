use anyhow::{Error, anyhow};
use serde::Serialize;
use std::env;

use crate::db::sqlite::Record;
use crate::embedding::vectorize::get_embedding;
use crate::ner::entities::get_entities;
use crate::ner::model::Entity;
use crate::rv::get::get_text;
use std::collections::HashMap;

use crate::ino_checker::name_checker::compare_names;
use crate::utils::funcs::{cosine_similarity, unordered_levenshtein};

use crate::db::sqlite as my_sqlite;

const MAX_DIS: usize = 7;
const MAX_TRESHOLD: f32 = 0.61;

pub struct RecordWithRelevance {
    pub record: my_sqlite::Record,
    pub similarity: f32,
}

#[derive(Serialize, Debug, Clone)]
struct Distances {
    #[serde(rename = "orig_name_dis")]
    not_normal_dis: usize,
    #[serde(rename = "norm_name_dis")]
    normal_dis: usize,
    #[serde(rename = "orig_name_dis_2")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name_dis: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Doc {
    pub status: String,
    pub name: String,
    pub is_removed: bool,
    pub similarity: f32,
    pub distance: usize,
    debug_distances: Distances,
}

#[derive(Debug, Serialize)]
pub struct WarningName {
    pub name: String,
    pub normal_name: String,
    pub context: String,
    pub name_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub docs: Vec<Doc>,
}

#[derive(Debug, Serialize)]
pub struct WarningNames {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<WarningName>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub accepted_names: Vec<WarningName>,
}

pub async fn get_inos_from_text(
    text: &str,
    inoagents: Vec<Record>,
    need_full_data: bool,
) -> Result<WarningNames, Error> {
    let entities = get_entities_list_with_retry(text, 3).await?;

    let mut inos: Vec<WarningName> = Vec::new();
    let mut accepted_names: Vec<WarningName> = Vec::new();

    for entity in entities {
        if entity.entity_type != "PER" && entity.entity_type != "ORG" {
            let name: WarningName = WarningName {
                name: entity.name,
                normal_name: entity.norm_name,
                context: entity.context,
                name_type: entity.entity_type,
                docs: Vec::new(),
            };
            accepted_names.push(name);
            continue;
        }
        let processed = get_most_relevant_names(MAX_TRESHOLD, MAX_DIS, &entity, &inoagents).await?;
        match processed {
            Some(ino) => inos.push(ino),
            None => {
                if need_full_data {
                    let most_relevant =
                        get_most_relevant_names(0.0, 100, &entity, &inoagents).await?;

                    if let Some(e) = most_relevant {
                        accepted_names.push(e)
                    }
                } else {
                    let name = WarningName {
                        name: entity.name,
                        normal_name: entity.norm_name,
                        context: entity.context,
                        name_type: entity.entity_type,
                        docs: Vec::new(),
                    };
                    accepted_names.push(name);
                }
            }
        }
    }

    Ok(WarningNames {
        warnings: inos,
        accepted_names,
    })
}

pub async fn get_inos(
    news_id: &str,
    inoagents: Vec<Record>,
    need_full_data: bool,
) -> Result<WarningNames, Error> {
    let text = get_text(news_id).await?;
    get_inos_from_text(&text, inoagents, need_full_data).await
}

async fn get_entities_list_with_retry(text: &str, max_retry: u8) -> Result<Vec<Entity>, Error> {
    let embeddibg_url = env::var("ENTITIES_URL").ok().unwrap();
    use tokio::time::{Duration, sleep};
    let mut last_error: Option<anyhow::Error> = None;
    for _ in 0..max_retry {
        match get_entities(text, &embeddibg_url).await {
            Ok(e) => return Ok(e.entities),
            Err(e) => {
                last_error = Some(e);
                sleep(Duration::from_millis(30)).await;
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown embedding error")))
}

async fn fetch_entity_embedding_with_retry(name: &str, max_retry: u8) -> Result<Vec<f32>, Error> {
    use tokio::time::{Duration, sleep};

    let mut last_error: Option<Error> = None;

    for _ in 0..max_retry {
        match get_entity_embedding(name).await {
            Ok(e) => return Ok(e),
            Err(e) => {
                last_error = Some(e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown embedding error")))
}

fn process_docs(docs: Vec<Doc>) -> Vec<Doc> {
    let mut grouped: HashMap<String, Vec<Doc>> = HashMap::new();

    // 1. Разбиваем по status
    for doc in docs {
        grouped.entry(doc.status.clone()).or_default().push(doc);
    }

    let mut result = Vec::new();

    // 2. Сортируем каждую группу и 3. Берем первый элемент
    for (_, mut group) in grouped {
        group.sort_by_key(|d| d.distance);
        if let Some(first) = group.into_iter().next() {
            result.push(first);
        }
    }

    result
}

async fn get_entity_embedding(name: &str) -> Result<Vec<f32>, Error> {
    let yandex_token = env::var("YANDEX_SECRET").ok().unwrap();
    let yandex_model = env::var("YANDEX_MODEL").ok().unwrap();
    let yandex_url = env::var("YANDEX_URL").ok().unwrap();

    let response = get_embedding(
        &name.to_lowercase(),
        &yandex_model,
        &yandex_token,
        &yandex_url,
    )
    .await?;

    if let Some(err_msg) = response.error {
        return Err(anyhow!("Embedding service error: {err_msg}"));
    }

    response
        .embedding
        .ok_or_else(|| anyhow!("No embedding returned for {name}"))
}

async fn get_most_relevant_names(
    treshold: f32,
    max_distance: usize,
    entity: &Entity,
    inoagents: &[Record],
) -> Result<Option<WarningName>, Error> {
    let embedding = fetch_entity_embedding_with_retry(&entity.name, 3).await?;

    let most_relevant = get_must_relevant(&embedding, inoagents, 5, treshold);
    if most_relevant.is_empty() {
        return Ok(None);
    }
    let mut docs: Vec<Doc> = Vec::new();
    for ag in most_relevant {
        let mut dis = unordered_levenshtein(
            &entity.norm_name.to_lowercase(),
            &ag.record.name.to_lowercase(),
        );
        let mut dis2 =
            unordered_levenshtein(&entity.name.to_lowercase(), &ag.record.name.to_lowercase());

        let mut distances = Distances {
            not_normal_dis: dis2,
            normal_dis: dis,
            name_dis: None,
        };

        if entity.entity_type == "PER" {
            let name_dis = compare_names(
                &entity.name.to_lowercase(),
                &ag.record.name.to_ascii_lowercase(),
            );

            if entity.name.contains(".") {
                distances.name_dis = Some(name_dis);
                dis = std::cmp::min(dis, name_dis);
                dis2 = std::cmp::min(dis2, name_dis)
            }
        }

        dis = std::cmp::min(dis, dis2);

        if dis <= max_distance {
            let doc = Doc {
                name: ag.record.name.clone(),
                is_removed: ag.record.is_removed,
                status: ag.record.record_type.clone(),
                similarity: ag.similarity,
                distance: dis,
                debug_distances: distances.clone(),
            };
            docs.push(doc);
        }
    }

    if docs.is_empty() {
        Ok(None)
    } else {
        let status_docs = process_docs(docs);
        let ino = WarningName {
            name: entity.name.clone(),
            normal_name: entity.norm_name.clone(),
            context: entity.context.clone(),
            name_type: entity.entity_type.clone(),
            docs: status_docs,
        };
        Ok(Some(ino))
    }
}

pub fn get_must_relevant(
    name: &[f32],
    inoagents: &[my_sqlite::Record],
    number: usize,
    treshold: f32,
) -> Vec<RecordWithRelevance> {
    let mut filtered_with_relevance: Vec<RecordWithRelevance> = Vec::new();
    for agent in inoagents {
        let sim = cosine_similarity(name, &agent.embedding);
        if sim >= treshold {
            let op = RecordWithRelevance {
                record: agent.clone(),
                similarity: sim,
            };
            filtered_with_relevance.push(op);
        }
    }
    filtered_with_relevance.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    filtered_with_relevance.truncate(number);
    filtered_with_relevance
}
