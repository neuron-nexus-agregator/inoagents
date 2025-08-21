use crate::ino_checker::utils::unordered_levenshtein;
use regex::Regex;

/// Нормализует текст: нижний регистр, удаление точек, кавычек, лишних пробелов
fn normalize_text(s: &str) -> String {
    s.to_lowercase()
        .replace(&['.', '"'][..], "")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Простая нормализация русских фамилий, убираем окончания склонений
fn normalize_surname(s: &str) -> String {
    if s.ends_with("ой") || s.ends_with("ая") || s.ends_with("ой") || s.ends_with("овой")
    {
        return s[..s.len() - 2].to_string();
    }
    s.to_string()
}

/// Извлекает псевдонимы из кавычек
fn extract_aliases(s: &str) -> Vec<String> {
    let re = Regex::new(r#""([^"]+)""#).unwrap();
    re.captures_iter(s).map(|cap| cap[1].to_string()).collect()
}

/// Выделяет фамилию (последнее слово) и инициалы (первые буквы остальных слов)
fn split_name(s: &str) -> (String, String) {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return ("".to_string(), "".to_string());
    }
    let surname = normalize_surname(parts.last().unwrap());
    let initials: String = parts
        .iter()
        .take(parts.len() - 1)
        .filter_map(|w| w.chars().next())
        .collect();
    (surname, initials)
}

/// Сравнивает два имени с учетом фамилии, инициалов и псевдонимов
pub fn compare_names(name_text: &str, name_registry: &str) -> usize {
    let text_norm = normalize_text(name_text);
    let registry_norm = normalize_text(name_registry);

    // Основная фамилия из реестра
    let registry_parts: Vec<&str> = registry_norm.split_whitespace().collect();
    let main_surname = registry_parts.first().unwrap_or(&"");

    // Псевдонимы
    let aliases = extract_aliases(name_registry);
    let mut all_registry_surnames = vec![normalize_surname(main_surname)];
    all_registry_surnames.extend(aliases.iter().map(|a| split_name(a).0));

    let (text_surname, text_initials) = split_name(&text_norm);

    // Находим минимальное расстояние по фамилии + инициалам
    let mut min_distance = usize::MAX;

    for surname in all_registry_surnames {
        let surname_dist = unordered_levenshtein(&text_surname, &surname);
        let initials_dist = unordered_levenshtein(&text_initials, &split_name(name_registry).1);
        let total = surname_dist + initials_dist;
        if total < min_distance {
            min_distance = total;
        }
    }

    min_distance
}
