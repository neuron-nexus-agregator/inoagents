use crate::embedding::model::Response;

pub trait Embedding {
    /// Получение векторного представления текста в виде 256-мерного массива f32
    async fn get_embedding(&self, text: &str) -> Result<Response, reqwest::Error>;
}
