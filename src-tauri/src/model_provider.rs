use crate::function_provider::FunctionDefinition;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LLMOptions {
    pub num_gpu: Option<u32>,
    pub num_ctx: Option<u32>,
    pub functions: Option<Vec<String>>,
    pub stream: bool,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn get_installed_models(&self) -> Result<Vec<String>, String>;
    async fn run_model(
        &self,
        app: AppHandle,
        model: String,
        messages: Vec<serde_json::Value>,
        options: Option<LLMOptions>,
        chat_id: String,
    ) -> Result<(), String>;

    async fn download_model(&self, app: tauri::AppHandle, model: String) -> Result<(), String>;
    async fn delete_model(&self, model: String) -> Result<(), String>;

    async fn stop_model(&self) -> Result<(), String>;
}
