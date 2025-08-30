use crate::db::model::Record;
use crate::ino_checker::model;

pub trait BasicChecker {
    /// Получение списка запрещенных имен по тексту
    async fn get_inos_from_text(
        &self,
        text: &str,
        need_full_data: bool,
    ) -> Result<model::WarningNames, anyhow::Error>;

    /// Получение списка запрещенных имен по id
    ///
    /// Внутри вызывает `get_inos_from_text()` после парсинга текста
    async fn get_inos(
        &self,
        news_id: &str,
        need_full_data: bool,
    ) -> Result<model::WarningNames, anyhow::Error>;

    /// Изменение списка запрещенных имен
    async fn change_warning_names(&self, new_warning_names: Vec<Record>);

    /// Добавление списка запрещенных имен
    async fn add_warning_names(&self, new_warning_names: Vec<Record>);
}

pub trait SmartNameChecker {
    /// Сравнение двух имен – используется при наличии инициалов в тексте.
    /// Например: `И.И. Иванов`
    fn compare_names(&self, name_text: &str, name_registry: &str) -> usize;
}
