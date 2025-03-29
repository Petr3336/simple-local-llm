use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::model_provider::{LLMOptions, ModelProvider};

use log::{info, debug, warn, error}; // [log]

pub struct OllamaProvider {
    stop_flag: Arc<AtomicBool>,
}

#[derive(Serialize)]
struct ModelRequest {
    name: String,
}

impl OllamaProvider {
    pub fn new() -> Self {
        info!("Initializing OllamaProvider"); // [log]
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
        info!("Fetching list of installed models from Ollama"); // [log]

        let client = Client::new();
        let response = client
            .get("http://localhost:11434/api/tags")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to /api/tags: {}", e); // [log]
                e.to_string()
            })?;

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
            .map_err(|e| {
                error!("Failed to parse JSON response: {}", e); // [log]
                format!("Ошибка парсинга JSON: {}", e)
            })?;

        let model_names: Vec<String> = tag_response
            .models
            .into_iter()
            .map(|tag| tag.name)
            .collect();

        debug!("Installed models: {:?}", model_names); // [log]
        Ok(model_names)
    }

    async fn run_model(
        &self,
        app: AppHandle,
        model: String,
        prompt: String,
        options: Option<LLMOptions>,
    ) -> Result<(), String> {
        info!("Running model '{}' with prompt: {:?}", model, prompt); // [log]

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
        debug!("Sending request to /api/generate with body: {:?}", body); // [log]

        let response = client
            .post("http://localhost:11434/api/generate")
            .json(&Value::Object(body))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to /api/generate: {}", e); // [log]
                e.to_string()
            })?;

        let mut stream = response.bytes_stream();
        let stop_flag = self.stop_flag.clone();

        tauri::async_runtime::spawn(async move {
            info!("Started receiving streamed response from model"); // [log]
            while let Some(chunk_result) = stream.next().await {
                if stop_flag.load(Ordering::SeqCst) {
                    info!("Model execution was stopped by user"); // [log]
                    let _ = app.emit("model-stopped", ());
                    break;
                }

                if let Ok(chunk_bytes) = chunk_result {
                    if let Ok(text) = std::str::from_utf8(&chunk_bytes) {
                        debug!("Received output chunk: {:?}", text); // [log]
                        let _ = app.emit("model-output", text.to_string());
                    } else {
                        warn!("Received non-UTF8 chunk from model"); // [log]
                    }
                } else {
                    warn!("Error reading chunk from model stream"); // [log]
                }
            }
            info!("Streaming from model has ended"); // [log]
        });

        Ok(())
    }

    async fn download_model(&self, app: tauri::AppHandle, model: String) -> Result<(), String> {
        info!("Requesting download for model: {}", model); // [log]

        let client = Client::new();
        let body = ModelRequest { name: model.clone() };

        let response = client
            .post("http://localhost:11434/api/pull")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Download request failed: {}", e); // [log]
                e.to_string()
            })?;

        if !response.status().is_success() {
            let msg = response.text().await.unwrap_or_default();
            error!("Model download failed: {}", msg); // [log]
            return Err(format!("Ошибка при загрузке модели: {}", msg));
        }

        info!("Model '{}' downloaded successfully", model); // [log]
        Ok(())
    }

    async fn delete_model(&self, model: String) -> Result<(), String> {
        info!("Requesting deletion of model: {}", model); // [log]

        let client = Client::new();
        let body = ModelRequest { name: model.clone() };

        let response = client
            .delete("http://localhost:11434/api/delete")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Delete request failed: {}", e); // [log]
                e.to_string()
            })?;

        if !response.status().is_success() {
            let msg = response.text().await.unwrap_or_default();
            error!("Model deletion failed: {}", msg); // [log]
            return Err(format!("Ошибка при удалении модели: {}", msg));
        }

        info!("Model '{}' deleted successfully", model); // [log]
        Ok(())
    }

    async fn stop_model(&self) -> Result<(), String> {
        self.stop_flag.store(true, Ordering::SeqCst);
        info!("Stop flag set for Ollama model"); // [log]
        Ok(())
    }
}
