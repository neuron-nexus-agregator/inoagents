use crate::db::model::Record;
use crate::ino_checker::model;

pub trait BasicChecker {
    async fn get_inos_from_text(
        &self,
        text: &str,
        need_full_data: bool,
    ) -> Result<model::WarningNames, anyhow::Error>;

    async fn get_inos(
        &self,
        news_id: &str,
        need_full_data: bool,
    ) -> Result<model::WarningNames, anyhow::Error>;

    fn change_warning_names(&mut self, new_warning_names: Vec<Record>);
    fn add_warning_names(&mut self, new_warning_names: Vec<Record>);
}

pub trait SmartNameChecker {
    fn compare_names(&self, name_text: &str, name_registry: &str) -> usize;
}
