mod db;
mod embedding;
mod ino_loader;
mod ner;
mod rv;

//use ino_loader::loader::load;

use dotenv::dotenv;

use std::collections::HashSet;
use strsim::levenshtein;

use std::env;
use tokio::time::{Duration, sleep};

use crate::db::sqlite as my_sqlite;
use crate::embedding::vectorize::get_embedding;
use crate::ner::entities::get_entities;

use crate::rv::get;

#[tokio::main]
async fn main() {
    dotenv().ok();

    //let res = load("assets/export.xlsx", "ino").ok();

    let id = "349703";

    let yandex_token = env::var("YANDEX_SECRET").ok().unwrap();
    let yandex_model = env::var("YANDEX_MODEL").ok().unwrap();
    let yandex_url = env::var("YANDEX_URL").ok().unwrap();
    let embeddibg_url = env::var("ENTITIES_URL").ok().unwrap();

    let db = my_sqlite::Database::new("assets/db/db.sqlite");

    match db {
        Err(e) => {
            eprintln!("{e}");
        }
        Ok(db) => {
            let inoagents = db.get_all().ok().unwrap();
            let text = get::getText(id).await;
            match text {
                Err(e) => eprintln!("{e}"),
                Ok(text) => {
                    let entities = get_entities(&text, &embeddibg_url).await;
                    match entities {
                        Err(e) => eprintln!("{e}"),
                        Ok(response) => {
                            let entities = response.entities;
                            for entity in entities.iter() {
                                let name = entity.name.clone();
                                let entity_type = entity.entity_type.clone();
                                if entity_type == "PER" || entity_type == "ORG" {
                                    let embedding = get_embedding(
                                        &name,
                                        &yandex_model,
                                        &yandex_token,
                                        &yandex_url,
                                    )
                                    .await
                                    .unwrap()
                                    .embedding
                                    .unwrap();

                                    for agent in inoagents.iter() {
                                        let sim = cosine_similarity(
                                            embedding.clone(),
                                            agent.embedding.clone(),
                                        );
                                        if sim >= 0.8 && !agent.is_removed {
                                            let lev = unordered_levenshtein(&name, &agent.name);
                                            println!(
                                                "{name} - иноагент по совпадению с {} и схожестью {}% и расстоянием Левенштейна {lev}",
                                                agent.name,
                                                sim * 100.0
                                            );
                                            break;
                                        }
                                    }
                                    sleep(Duration::from_secs(1)).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cosine_similarity(v1: Vec<f32>, v2: Vec<f32>) -> f32 {
    if v1.len() != v2.len() {
        return -1.0;
    }

    let mut up = 0.0;
    let mut down_a = 0.0;
    let mut down_b = 0.0;

    for i in 0..v1.len() {
        let a = v1[i];
        let b = v2[i];

        up += a * b;
        down_a += a * a;
        down_b += b * b;
    }

    up / (down_a.sqrt() * down_b.sqrt())
}

fn unordered_levenshtein(s1: &str, s2: &str) -> usize {
    // Разбиваем строки на слова и собираем в множества
    let words1: HashSet<&str> = s1.split_whitespace().collect();
    let words2: HashSet<&str> = s2.split_whitespace().collect();

    // Считаем количество несовпадающих слов
    let only_in_1 = words1.difference(&words2).count();
    let only_in_2 = words2.difference(&words1).count();

    // Можно добавить «взвешенный» вариант с Левенштейном на словах
    let mut levenshtein_sum = 0;
    for w1 in &words1 {
        // Находим минимальное расстояние Левенштейна до любого слова в другой строке
        let min_dist = words2
            .iter()
            .map(|w2| levenshtein(w1, w2))
            .min()
            .unwrap_or(w1.len());
        levenshtein_sum += min_dist;
    }

    // Итоговое расстояние — сумма несовпадающих слов и расстояний между ними
    levenshtein_sum + only_in_1 + only_in_2
}

// INIT INOAGENTS
// for item in res.unwrap().iter() {
//     loop {
//         let emb_model =
//             get_embedding(&item.name, &yandex_model, &yandex_token, &yandex_url).await;

//         match emb_model {
//             Err(e) => {
//                 eprintln!(
//                     "Ошибка для '{}': {}. Повтор через 1 секунду...",
//                     item.name, e
//                 );
//                 sleep(Duration::from_secs(1)).await;
//                 continue; // пробуем снова тот же item
//             }
//             Ok(emb) => {
//                 if let Some(t) = emb.embedding {
//                     println!("{} - {}", item.name, t.len());
//                     let record = my_sqlite::Record {
//                         name: item.name.to_string(),
//                         record_type: "ino".to_string(),
//                         embedding: t,
//                         is_removed: false,
//                     };
//                     match db.insert(&record) {
//                         Ok(_) => {
//                             println!("Успешно записано: {}", item.name)
//                         }
//                         Err(e) => eprintln!("{e}"),
//                     }
//                     break; // успех, переходим к следующему item
//                 } else if let Some(eq) = emb.error {
//                     println!("{} - {}", item.name, eq);
//                     break; // API вернул ошибку, но не исключение — пропускаем элемент
//                 }
//             }
//         }
//     }
// }
