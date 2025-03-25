use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::model_provider::{LLMOptions, ModelProvider};

pub struct OllamaProvider {
    stop_flag: Arc<AtomicBool>,
}

#[derive(Serialize)]
struct ModelRequest {
    name: String,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    async fn get_installed_models(&self) -> Result<Vec<String>, String> {
        let client = Client::new();
        let response = client
            .get("http://localhost:11434/api/tags")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        #[derive(Deserialize)]
        struct TagResponse {
            models: Vec<ModelTag>,
        }

        #[derive(Deserialize)]
        struct ModelTag {
            name: String,
        }

        let tag_response: TagResponse = response
            .json()
            .await
            .map_err(|e| format!("Ошибка парсинга JSON: {}", e))?;

        Ok(tag_response
            .models
            .into_iter()
            .map(|tag| tag.name)
            .collect())
    }

    async fn run_model(
        &self,
        app: AppHandle,
        model: String,
        prompt: String,
        options: Option<LLMOptions>,
    ) -> Result<(), String> {
        let client = Client::new();
    
        let mut body = serde_json::Map::new();
        body.insert("model".to_string(), json!(model));
        body.insert("prompt".to_string(), json!(prompt));
        body.insert("stream".to_string(), json!(true));
        if let Some(opts) = options {
            if let Some(num_gpu) = opts.num_gpu {
                body.insert("num_gpu".to_string(), json!(num_gpu));
            }
            if let Some(num_ctx) = opts.num_ctx {
                body.insert("num_ctx".to_string(), json!(num_ctx));
            }
        }
    
        self.stop_flag.store(false, Ordering::SeqCst);
    
        let response = client
            .post("http://localhost:11434/api/generate")
            .json(&Value::Object(body))
            .send()
            .await
            .map_err(|e| e.to_string())?;
    
        let mut stream = response.bytes_stream();
        let stop_flag = self.stop_flag.clone();
    
        tauri::async_runtime::spawn(async move {
            while let Some(chunk_result) = stream.next().await {
                if stop_flag.load(Ordering::SeqCst) {
                    let _ = app.emit("model-stopped", ());
                    break;
                }
    
                if let Ok(chunk_bytes) = chunk_result {
                    if let Ok(text) = std::str::from_utf8(&chunk_bytes) {
                        let _ = app.emit("model-output", text.to_string());
                    }
                }
            }
        });
    
        Ok(())
    }
    

    async fn download_model(&self, model: String) -> Result<(), String> {
        let client = Client::new();
        let body = ModelRequest { name: model };

        let response = client
            .post("http://localhost:11434/api/pull")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!(
                "Ошибка при загрузке модели: {}",
                response.text().await.unwrap_or_default()
            ));
        }

        Ok(())
    }

    async fn delete_model(&self, model: String) -> Result<(), String> {
        let client = Client::new();
        let body = ModelRequest { name: model };

        let response = client
            .delete("http://localhost:11434/api/delete")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!(
                "Ошибка при удалении модели: {}",
                response.text().await.unwrap_or_default()
            ));
        }

        Ok(())
    }

    async fn stop_model(&self) -> Result<(), String> {
        self.stop_flag.store(true, Ordering::SeqCst);
        Ok(())
    }
}
