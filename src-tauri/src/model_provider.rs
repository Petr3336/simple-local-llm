use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use crate::function_provider::FunctionDefinition;

/// Настройки запуска LLM модели.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LLMOptions {
    /// Количество потоков GPU (если поддерживается).
    pub num_gpu: Option<u32>,
    /// Контекстное окно модели.
    pub num_ctx: Option<u32>,
    /// Имена функций, доступных модели.
    pub functions: Option<Vec<String>>,
    /// Включить потоковый вывод.
    pub stream: bool
}

/// Интерфейс (трейт) для провайдеров LLM.
///
/// Все реализации должны уметь запускать, останавливать и управлять моделями.
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Имя провайдера (например, "ollama", "llama_cpp").
    fn name(&self) -> &'static str;

    /// Получить список установленных моделей.
    async fn get_installed_models(&self) -> Result<Vec<String>, String>;

    /// Запустить модель с заданными сообщениями и параметрами.
    async fn run_model(
        &self,
        app: AppHandle,
        model: String,
        messages: Vec<serde_json::Value>,
        options: Option<LLMOptions>,
        chat_id: String
    ) -> Result<(), String>;

    /// Скачать модель.
    async fn download_model(&self, app: tauri::AppHandle, model: String) -> Result<(), String>;

    /// Удалить модель.
    async fn delete_model(&self, model: String) -> Result<(), String>;

    /// Остановить работу модели.
    async fn stop_model(&self) -> Result<(), String>;
}
