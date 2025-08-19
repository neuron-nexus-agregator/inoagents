use crate::db::sqlite as my_sqlite;
use std::collections::HashSet;
use strsim::levenshtein;

pub fn get_must_relevant(
    name: &[f32],
    inoagents: &[my_sqlite::Record],
    number: usize,
    treshold: f32,
) -> Vec<my_sqlite::Record> {
    // Фильтруем записи по порогу сходства
    let mut filtered: Vec<my_sqlite::Record> = inoagents
        .iter()
        .filter(|rec| cosine_similarity(name, &rec.embedding) >= treshold)
        .cloned()
        .collect();

    // Сортируем по убыванию сходства
    filtered.sort_by(|a, b| {
        let sim_a = cosine_similarity(name, &a.embedding);
        let sim_b = cosine_similarity(name, &b.embedding);
        sim_b
            .partial_cmp(&sim_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Обрезаем до нужного количества
    filtered.truncate(number);
    filtered
}

fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
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

pub fn unordered_levenshtein(s1: &str, s2: &str) -> usize {
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
