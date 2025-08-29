use std::collections::HashSet;
use strsim::levenshtein;

pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    if v1.len() != v2.len() || v1.is_empty() || v2.is_empty() {
        return -1.0;
    }

    let mut up = 0.0;
    let mut down_a = 0.0;
    let mut down_b = 0.0;

    for (&a, &b) in v1.iter().zip(v2.iter()) {
        up += a * b;
        down_a += a * a;
        down_b += b * b;
    }

    up / (down_a.sqrt() * down_b.sqrt())
}

pub fn unordered_levenshtein(s1: &str, s2: &str) -> usize {
    let s1_norm = s1.to_lowercase();
    let s2_norm = s2.to_lowercase();

    // Разбиваем строки на слова и собираем в множества
    let words1: HashSet<&str> = s1_norm.split_whitespace().collect();
    let words2: HashSet<&str> = s2_norm.split_whitespace().collect();

    const MULTIPLIER: usize = 2;

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
    levenshtein_sum + (only_in_1 + only_in_2) * MULTIPLIER
}

pub fn keep_russian_and_dot(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            // Русские буквы (А-Я, а-я, Ё, ё)
            (*c >= 'А' && *c <= 'я') || *c == 'Ё' || *c == 'ё'
            // Английские буквы (A-Z, a-z)
            || (*c >= 'A' && *c <= 'Z') || (*c >= 'a' && *c <= 'z')
            // Символы '.' и '&'
            || *c == '.' || *c == '&'
        })
        .collect()
}
