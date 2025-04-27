use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::model_provider::{LLMOptions, ModelProvider};

use log::{debug, error, info, warn}; // [log]

use crate::function_provider::{FunctionDefinition, LlmFunction};
use crate::initialize_functions;
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

        let tag_response: TagResponse = response.json().await.map_err(|e| {
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
        messages: Vec<serde_json::Value>,
        options: Option<LLMOptions>,
        chat_id: String,
    ) -> Result<(), String> {
        info!("Running model '{}' with messages: {:?}", model, messages);

        let client = reqwest::Client::new();
        let mut body = serde_json::Map::new();

        body.insert("model".to_string(), json!(model));
        body.insert("messages".to_string(), json!(messages));

        // Определяем, включён ли стриминг
        let stream_enabled = options.as_ref().map(|opts| opts.stream).unwrap_or(false);
        body.insert("stream".to_string(), json!(stream_enabled));

        let all_functions = initialize_functions();
        let mut tools_payload = Vec::new();
        let mut callable_map: HashMap<String, Arc<dyn LlmFunction>> = HashMap::new();
        let mut use_tools = false;

        if let Some(opts) = options {
            if let Some(num_gpu) = opts.num_gpu {
                body.insert("num_gpu".to_string(), json!(num_gpu));
            }
            if let Some(num_ctx) = opts.num_ctx {
                body.insert("num_ctx".to_string(), json!(num_ctx));
            }
            if let Some(enabled) = &opts.functions {
                for name in enabled {
                    if let Some(func) = all_functions.iter().find(|f| f.definition().name == *name)
                    {
                        let def = func.definition();

                        let properties = def
                            .parameters
                            .iter()
                            .map(|(key, param)| {
                                let mut prop = json!({ "type": param.param_type });
                                prop["description"] = json!(param.description);
                                (key.clone(), prop)
                            })
                            .collect::<serde_json::Map<_, _>>();

                        let required = def.parameters.keys().cloned().collect::<Vec<_>>();

                        let tool = json!({
                            "type": "function",
                            "function": {
                                "name": def.name,
                                "description": def.description.unwrap_or_default(),
                                "parameters": {
                                    "type": "object",
                                    "properties": properties,
                                    "required": required
                                }
                            }
                        });

                        tools_payload.push(tool);
                        callable_map.insert(def.name.clone(), func.clone());
                        use_tools = true;
                    }
                }
            }
        }

        if use_tools {
            body.insert("tools".into(), json!(tools_payload));
        }

        self.stop_flag.store(false, Ordering::SeqCst);
        debug!("Sending request to /api/chat with body: {:?}", body);

        let response = client
            .post("http://localhost:11434/api/chat")
            .json(&serde_json::Value::Object(body))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to /api/chat: {}", e);
                e.to_string()
            })?;

        let mut stream = response.bytes_stream();
        let stop_flag = self.stop_flag.clone();
        let chat_id_clone = chat_id.clone();

        tauri::async_runtime::spawn(async move {
            info!("Started receiving response from model");
            if stream_enabled {
                // Стриминг включён: эмитим каждую порцию через "model-stream-output"
                while let Some(chunk_result) = stream.next().await {
                    if stop_flag.load(Ordering::SeqCst) {
                        info!("Model execution was stopped by user");
                        let payload = json!({ "chat_id": chat_id_clone }).to_string();
                        let _ = app.emit("model-stopped", payload);
                        break;
                    }
                    if let Ok(chunk_bytes) = chunk_result {
                        if let Ok(text) = std::str::from_utf8(&chunk_bytes) {
                            debug!("Received output chunk: {:?}", text);
                            let payload = json!({
                                "chat_id": chat_id_clone,
                                "output": text
                            })
                            .to_string();
                            let _ = app.emit("model-stream-output", payload);
                        } else {
                            warn!("Received non-UTF8 chunk from model");
                        }
                    } else {
                        warn!("Error reading chunk from model stream");
                    }
                }
            } else {
                // Стриминг отключён: накапливаем ответ и эмитим его через "model-output"
                let mut full_response = String::new();
                while let Some(chunk_result) = stream.next().await {
                    if stop_flag.load(Ordering::SeqCst) {
                        info!("Model execution was stopped by user");
                        let payload = json!({ "chat_id": chat_id_clone }).to_string();
                        let _ = app.emit("model-stopped", payload);
                        break;
                    }
                    if let Ok(chunk_bytes) = chunk_result {
                        if let Ok(text) = std::str::from_utf8(&chunk_bytes) {
                            debug!("Received output chunk: {:?}", text);
                            full_response.push_str(text);
                        } else {
                            warn!("Received non-UTF8 chunk from model");
                        }
                    } else {
                        warn!("Error reading chunk from model stream");
                    }
                }

                // Пробуем преобразовать полный ответ в JSON-объект
                let output_value = match serde_json::from_str::<serde_json::Value>(&full_response) {
                    Ok(parsed) => parsed,
                    Err(_) => json!(full_response),
                };

                // Проверяем, есть ли вызов функции в ответе
                if let Some(message) = output_value.get("message") {
                    if let Some(tool_calls) = message.get("tool_calls").and_then(|v| v.as_array()) {
                        if let Some(first_tool_call) = tool_calls.first() {
                            if let Some(function) = first_tool_call.get("function") {
                                if let (Some(name), Some(arguments)) = (
                                    function.get("name").and_then(|v| v.as_str()),
                                    function.get("arguments").cloned(),
                                ) {
                                    info!("Received function call: {}", name);
                                    if let Some(func) = callable_map.get(name) {
                                        match func.call(arguments).await {
                                            Ok(result) => {
                                                // Отправляем результат вызова функции
                                                let tool_response = json!({
                                                    "chat_id": chat_id_clone,
                                                    "output": {
                                                        "role": "tool",
                                                        "content": format!("Результат выполнения функции {name}: {result}"),
                                                        "tool_call_id": name
                                                    }
                                                });
                                                let _ = app.emit(
                                                    "model-output",
                                                    tool_response.to_string(),
                                                );
                                            }
                                            Err(err) => {
                                                let _ = app.emit(
                                                    "function-error",
                                                    format!("{}: {}", name, err),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Если нет вызова функции, отправляем обычный ответ
                            let payload = json!({
                                "chat_id": chat_id_clone,
                                "output": output_value
                            });
                            let _ = app.emit("model-output", payload.to_string());
                        }
                    } else {
                        // Если нет вызова функции, отправляем обычный ответ
                        let payload = json!({
                            "chat_id": chat_id_clone,
                            "output": output_value
                        });
                        let _ = app.emit("model-output", payload.to_string());
                    }
                }
            }
            info!("Response from model has ended");
        });

        Ok(())
    }

    /* async fn run_model(
        &self,
        app: AppHandle,
        model: String,
        messages: Vec<Value>, // теперь принимаем массив сообщений
        options: Option<LLMOptions>,
        chat_id: String,      // дополнительный параметр для идентификации чата
    ) -> Result<(), String> {
        info!(
            "Running model '{}' for chat_id {} with {} messages",
            model,
            chat_id,
            messages.len()
        );

        let client = Client::new();
        let mut tools_payload = Vec::new();
        let mut callable_map: HashMap<String, Arc<dyn LlmFunction>> = HashMap::new();
        let all_functions = initialize_functions();

        let mut use_tools = false;
        if let Some(opts) = &options {
            if let Some(enabled) = &opts.functions {
                for name in enabled {
                    if let Some(func) = all_functions.iter().find(|f| f.definition().name == *name) {
                        let def = func.definition();

                        let properties = def.parameters.iter().map(|(key, param)| {
                            let mut prop = json!({ "type": param.param_type });
                            prop["description"] = json!(param.description);
                            (key.clone(), prop)
                        }).collect::<serde_json::Map<_, _>>();

                        let required = def.parameters.keys().cloned().collect::<Vec<_>>();

                        let tool = json!({
                            "type": "function",
                            "function": {
                                "name": def.name,
                                "description": def.description.unwrap_or_default(),
                                "parameters": {
                                    "type": "object",
                                    "properties": properties,
                                    "required": required
                                }
                            }
                        });

                        tools_payload.push(tool);
                        callable_map.insert(def.name.clone(), func.clone());
                        use_tools = true;
                    }
                }
            }
        }

        let mut body = serde_json::Map::new();
        body.insert("model".into(), json!(model));
        body.insert("messages".into(), json!(messages));
        // Если используются функции, stream отключаем (false), иначе stream:true
        body.insert("stream".into(), json!(!use_tools));
        if use_tools {
            body.insert("tools".into(), json!(tools_payload));
        }

        self.stop_flag.store(false, Ordering::SeqCst);
        debug!("Sending request to /api/chat with body: {:#}", json!(body));

        let mut response = client
            .post("http://localhost:11434/api/chat")
            .json(&Value::Object(body))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to /api/chat: {}", e);
                e.to_string()
            })?;

        // Если используются функции, обрабатываем весь ответ сразу
        if use_tools {
            let response_text = response.text().await.map_err(|e| e.to_string())?;
            debug!("Got full response: {}", response_text);

            let json_resp: Value =
                serde_json::from_str(&response_text).map_err(|e| e.to_string())?;

            // Если имеется вызов функции
            if let Some(func_call) = json_resp.get("message").and_then(|m| m.get("function_call"))
            {
                if let (Some(name), Some(arguments), Some(tool_call_id)) = (
                    func_call.get("name").and_then(|v| v.as_str()),
                    func_call.get("arguments").cloned(),
                    json_resp
                        .get("message")
                        .and_then(|m| m.get("tool_call_id"))
                        .and_then(|v| v.as_str()),
                ) {
                    info!("Received function_call: {}", name);
                    if let Some(func) = callable_map.get(name) {
                        match func.call(arguments).await {
                            Ok(result) => {
                                // Клонируем входящие сообщения и добавляем tool-response
                                let mut new_messages = messages.clone();
                                new_messages.push(json!({
                                    "role": "tool",
                                    "tool_call_id": tool_call_id,
                                    "content": result.to_string()
                                }));

                                // Отправляем второй запрос без tools
                                let mut new_body = serde_json::Map::new();
                                new_body.insert("model".into(), json!(model));
                                new_body.insert("messages".into(), json!(new_messages));
                                new_body.insert("stream".into(), json!(false));

                                let second_response = client
                                    .post("http://localhost:11434/api/chat")
                                    .json(&Value::Object(new_body))
                                    .send()
                                    .await
                                    .map_err(|e| e.to_string())?;

                                let second_text = second_response
                                    .text()
                                    .await
                                    .unwrap_or_default();
                                debug!(
                                    "Final response after tool_response: {}",
                                    second_text
                                );

                                let payload = json!({
                                    "chat_id": chat_id,
                                    "output": second_text
                                });
                                let _ = app.emit("model-output", payload.to_string());
                            }
                            Err(err) => {
                                let _ = app.emit("function-error", format!("{}: {}", name, err));
                            }
                        }
                    }
                }
            } else {
                // Если нет вызова функции — обычный ответ
                if let Some(msg) = json_resp.get("message") {
                    let payload = json!({
                        "chat_id": chat_id,
                        "output": msg.to_string()
                    });
                    let _ = app.emit("model-output", payload.to_string());
                }
            }
            return Ok(());
        }

        // Если функции не используются, обрабатываем потоковый ответ
        let mut stream = response.bytes_stream();
        let stop_flag = self.stop_flag.clone();

        tauri::async_runtime::spawn(async move {
            info!("Started receiving streamed response from model");

            while let Some(chunk_result) = stream.next().await {
                if stop_flag.load(Ordering::SeqCst) {
                    info!("Model execution was stopped by user");
                    let _ = app.emit("model-stopped", ());
                    break;
                }

                if let Ok(chunk_bytes) = chunk_result {
                    if let Ok(text) = std::str::from_utf8(&chunk_bytes) {
                        debug!("Received chunk: {:?}", text);
                        for line in text.lines() {
                            let payload = json!({
                                "chat_id": chat_id,
                                "output": line.to_string()
                            });
                            let _ = app.emit("model-stream-output", payload.to_string());
                        }
                    } else {
                        warn!("Received non-UTF8 chunk from model");
                    }
                } else {
                    warn!("Error reading chunk from model stream");
                }
            }

            info!("Streaming from model has ended");
        });

        Ok(())
    } */

    async fn download_model(&self, app: tauri::AppHandle, model: String) -> Result<(), String> {
        info!("Requesting download for model: {}", model); // [log]

        let client = Client::new();
        let body = ModelRequest {
            name: model.clone(),
        };

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
        let body = ModelRequest {
            name: model.clone(),
        };

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
