use crate::db::model::Record;
use crate::embedding::interface::Embedding;
use crate::ino_checker::ann::ANNIndex;
use crate::ino_checker::interface::{BasicChecker, SmartNameChecker};
use crate::ino_checker::model::{self, WarningName};
use crate::ner::interface::Entities;
use crate::ner::model::Entity;
use crate::rv::get::get_text;
use crate::utils::funcs::keep_russian_and_dot;
use crate::utils::funcs::{cosine_similarity, unordered_levenshtein};
use std::collections::HashMap;
use tokio::sync::Mutex;

use futures::future::join_all;
type TaskResult = Result<(Option<model::WarningName>, Option<model::WarningName>), anyhow::Error>;

const MAX_DIS: usize = 7;
const MAX_TRESHOLD: f32 = 0.61;

pub struct WarningNamesChecker<T: Embedding, S: SmartNameChecker, E: Entities> {
    warning_names: Mutex<Vec<Record>>,
    vectorizer: T,
    name_checker: S,
    entities: E,
    index: Mutex<ANNIndex>,
}

// basic public functions
impl<T: Embedding, S: SmartNameChecker, E: Entities> WarningNamesChecker<T, S, E> {
    pub fn new(warning_names: Vec<Record>, vectorizer: T, name_checker: S, entities: E) -> Self {
        let mut index = ANNIndex::new("warning-names", 256);
        println!("Adding name to index");
        let mut i = 1;
        for name in warning_names.clone() {
            println!("Adding name {i}/{}", warning_names.len());
            i += 1;
            index.add(name);
        }

        WarningNamesChecker {
            warning_names: Mutex::new(warning_names),
            vectorizer,
            name_checker,
            entities,
            index: Mutex::new(index),
        }
    }
}

// basic non-public functions
impl<T: Embedding, S: SmartNameChecker, E: Entities> WarningNamesChecker<T, S, E> {
    async fn get_must_relevant(
        &self,
        name: &[f32],
        number: usize,
        treshold: f32,
    ) -> Vec<model::RecordWithRelevance> {
        let mut filtered_with_relevance: Vec<model::RecordWithRelevance> = Vec::new();

        let warning_names = self.index.lock().await.search(name, number);

        for agent in warning_names {
            let sim = cosine_similarity(name, &agent.embedding);
            if sim >= treshold {
                let op = model::RecordWithRelevance {
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

    async fn get_most_relevant_names(
        &self,
        treshold: f32,
        max_distance: usize,
        entity: &Entity,
    ) -> Result<Option<model::WarningName>, anyhow::Error> {
        let name = keep_russian_and_dot(&entity.name);
        if name.is_empty() {
            return Ok(None);
        }

        let embedding = self.fetch_entity_embedding_with_retry(&name, 3).await?;

        let most_relevant = self.get_must_relevant(&embedding, 5, treshold).await;
        if most_relevant.is_empty() {
            return Ok(None);
        }
        let mut docs: Vec<model::Doc> = Vec::new();
        for ag in most_relevant {
            let mut dis = unordered_levenshtein(
                &entity.norm_name.to_lowercase(),
                &ag.record.name.to_lowercase(),
            );
            let mut dis2 =
                unordered_levenshtein(&entity.name.to_lowercase(), &ag.record.name.to_lowercase());

            let mut distances = model::Distances {
                not_normal_dis: dis2,
                normal_dis: dis,
                name_dis: None,
            };

            if entity.entity_type == "PER" {
                let name_dis = self.name_checker.compare_names(
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
                let doc = model::Doc {
                    name: ag.record.name.clone(),
                    is_removed: ag.record.is_removed,
                    status: ag.record.record_type.clone(),
                    similarity: ag.similarity,
                    distance: dis,
                    debug_distances: Some(distances.clone()),
                };
                docs.push(doc);
            }
        }

        if docs.is_empty() {
            Ok(None)
        } else {
            let status_docs = self.process_docs(docs);
            let ino = model::WarningName {
                name: entity.name.clone(),
                normal_name: entity.norm_name.clone(),
                context: entity.context.clone(),
                name_type: entity.entity_type.clone(),
                docs: status_docs,
            };
            Ok(Some(ino))
        }
    }

    fn process_docs(&self, docs: Vec<model::Doc>) -> Vec<model::Doc> {
        let mut grouped: HashMap<String, Vec<model::Doc>> = HashMap::new();

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

    async fn fetch_entity_embedding_with_retry(
        &self,
        name: &str,
        max_retry: u8,
    ) -> Result<Vec<f32>, anyhow::Error> {
        use tokio::time::{Duration, sleep};

        let mut last_error: Option<anyhow::Error> = None;

        for _ in 0..max_retry {
            match self.vectorizer.get_embedding(name).await {
                Ok(e) => {
                    if let Some(e) = e.error {
                        last_error = Some(anyhow::anyhow!("{e}"));
                    } else if let Some(e) = e.embedding {
                        return Ok(e);
                    } else {
                        last_error = Some(anyhow::anyhow!("Unknown embedding error"));
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("{e}"));
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown embedding error")))
    }

    async fn get_entities_list_with_retry(
        &self,
        text: &str,
        max_retry: u8,
    ) -> Result<Vec<Entity>, anyhow::Error> {
        //let embeddibg_url = env::var("ENTITIES_URL").ok().unwrap();
        use tokio::time::{Duration, sleep};
        let mut last_error: Option<anyhow::Error> = None;
        for _ in 0..max_retry {
            match self.entities.get_entities(text).await {
                Ok(e) => return Ok(e.entities),
                Err(e) => {
                    last_error = Some(e);
                    sleep(Duration::from_millis(30)).await;
                }
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown embedding error")))
    }

    async fn check_english_name(&self, entity: Entity) -> Option<WarningName> {
        let test_name = entity.name.clone().to_lowercase();
        let mut res = WarningName {
            name: entity.name.clone(),
            normal_name: entity.norm_name.clone(),
            name_type: entity.entity_type.clone(),
            context: entity.context.clone(),
            docs: Vec::new(),
        };
        for warning_name in self.warning_names.lock().await.clone() {
            let u_name = warning_name.name.to_lowercase();
            if u_name.contains(&test_name) {
                let doc = model::Doc {
                    status: warning_name.record_type.clone(),
                    similarity: 1.0,
                    distance: 0,
                    is_removed: warning_name.is_removed,
                    name: warning_name.name.clone(),
                    debug_distances: None,
                };
                res.docs.push(doc);
            }
        }
        if !res.docs.is_empty() {
            let new_docs = self.process_docs(res.docs.clone());
            res.docs = new_docs;
            return Some(res);
        }
        None
    }
}

// trait implementation
impl<T: Embedding, S: SmartNameChecker, E: Entities> BasicChecker for WarningNamesChecker<T, S, E> {
    async fn change_warning_names(&self, new_warning_names: Vec<Record>) {
        let mut guard = self.index.lock().await;
        let mut new_index = ANNIndex::new("warning_names", 256);
        println!("Adding name to index");
        let mut i = 1;
        let len = new_warning_names.len();
        for name in new_warning_names.clone() {
            println!("Adding name {i}/{len}");
            i += 1;
            new_index.add(name);
        }
        *guard = new_index;
    }

    async fn add_warning_names(&self, new_warning_names: Vec<Record>) {
        let mut w_guard = self.warning_names.lock().await;
        w_guard.extend(new_warning_names.clone());
        let mut idx_guard = self.index.lock().await;
        println!("Adding name to index");
        let mut i = 1;
        let len = new_warning_names.len();
        for name in new_warning_names {
            println!("Adding name {i}/{len}");
            i += 1;
            idx_guard.add(name);
        }
    }

    async fn get_inos_from_text(
        &self,
        text: &str,
        need_full_data: bool,
    ) -> Result<model::WarningNames, anyhow::Error> {
        let entities = self.get_entities_list_with_retry(text, 3).await?;

        // Собираем задачи
        let tasks = entities.into_iter().map(|entity| {
            let this = self;
            async move {
                if entity.entity_type != "PER" && entity.entity_type != "ORG" {
                    return Ok::<_, anyhow::Error>((
                        None,
                        Some(model::WarningName {
                            name: entity.name,
                            normal_name: entity.norm_name,
                            context: entity.context,
                            name_type: entity.entity_type,
                            docs: Vec::new(),
                        }),
                    ));
                }

                let processed = this
                    .get_most_relevant_names(MAX_TRESHOLD, MAX_DIS, &entity)
                    .await?;

                if let Some(ino) = processed {
                    return Ok((Some(ino), None));
                }

                if keep_russian_and_dot(&entity.name).is_empty() {
                    if let Some(e) = this.check_english_name(entity.clone()).await {
                        return Ok((Some(e), None));
                    } else if need_full_data {
                        return Ok((
                            None,
                            Some(model::WarningName {
                                name: entity.name.clone(),
                                normal_name: entity.norm_name.clone(),
                                context: entity.context.clone(),
                                name_type: entity.entity_type.clone(),
                                docs: Vec::new(),
                            }),
                        ));
                    }
                } else if need_full_data {
                    let most_relevant = this.get_most_relevant_names(0.0, 100, &entity).await?;
                    if let Some(e) = most_relevant {
                        return Ok((None, Some(e)));
                    }
                }

                Ok((
                    None,
                    Some(model::WarningName {
                        name: entity.name,
                        normal_name: entity.norm_name,
                        context: entity.context,
                        name_type: entity.entity_type,
                        docs: Vec::new(),
                    }),
                ))
            }
        });

        // Запускаем все параллельно
        let results: Vec<TaskResult> = join_all(tasks).await;

        let mut inos = Vec::new();
        let mut accepted_names = Vec::new();

        for res in results {
            let (ino, acc) = res?;
            if let Some(i) = ino {
                inos.push(i);
            }
            if let Some(a) = acc {
                accepted_names.push(a);
            }
        }

        Ok(model::WarningNames {
            warnings: inos,
            accepted_names,
        })
    }

    async fn get_inos(
        &self,
        news_id: &str,
        need_full_data: bool,
    ) -> Result<model::WarningNames, anyhow::Error> {
        let text = get_text(news_id).await?;
        self.get_inos_from_text(&text, need_full_data).await
    }
}
