use crate::ino_checker::interface::SmartNameChecker;
use crate::utils::funcs::unordered_levenshtein;
use regex::Regex;

pub struct NameChecker {}

impl NameChecker {
    pub fn new() -> Self {
        NameChecker {}
    }

    fn normalize_text(&self, s: &str) -> String {
        s.to_lowercase()
            .replace(&['.', '"'][..], "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Простая нормализация русских фамилий, убираем окончания склонений
    fn normalize_surname(&self, s: &str) -> String {
        if s.ends_with("ой") || s.ends_with("ая") || s.ends_with("ой") || s.ends_with("овой")
        {
            return s[..s.len() - 2].to_string();
        }
        s.to_string()
    }

    /// Извлекает псевдонимы из кавычек
    fn extract_aliases(&self, s: &str) -> Vec<String> {
        let re = Regex::new(r#""([^"]+)""#).unwrap();
        re.captures_iter(s).map(|cap| cap[1].to_string()).collect()
    }

    /// Выделяет фамилию (последнее слово) и инициалы (первые буквы остальных слов)
    fn split_name(&self, s: &str) -> (String, String) {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return ("".to_string(), "".to_string());
        }
        let surname = self.normalize_surname(parts.last().unwrap());
        let initials: String = parts
            .iter()
            .take(parts.len() - 1)
            .filter_map(|w| w.chars().next())
            .collect();
        (surname, initials)
    }
}

impl SmartNameChecker for NameChecker {
    fn compare_names(&self, name_text: &str, name_registry: &str) -> usize {
        let text_norm = self.normalize_text(name_text);
        let registry_norm = self.normalize_text(name_registry);

        // Основная фамилия из реестра
        let registry_parts: Vec<&str> = registry_norm.split_whitespace().collect();
        let main_surname = registry_parts.first().unwrap_or(&"");

        // Псевдонимы
        let aliases = self.extract_aliases(name_registry);
        let mut all_registry_surnames = vec![self.normalize_surname(main_surname)];
        all_registry_surnames.extend(aliases.iter().map(|a| self.split_name(a).0));

        let (text_surname, text_initials) = self.split_name(&text_norm);

        // Находим минимальное расстояние по фамилии + инициалам
        let mut min_distance = usize::MAX;

        for surname in all_registry_surnames {
            let surname_dist = unordered_levenshtein(&text_surname, &surname);
            let initials_dist =
                unordered_levenshtein(&text_initials, &self.split_name(name_registry).1);
            let total = surname_dist + initials_dist;
            if total < min_distance {
                min_distance = total;
            }
        }

        min_distance
    }
}
