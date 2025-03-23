// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::Emitter;

#[derive(Deserialize, Serialize)]
pub struct LLMOptions {
    pub num_gpu: Option<u32>,
    pub num_ctx: Option<u32>,
    // Можно добавить другие параметры по необходимости
}

#[tauri::command]
async fn run_ollama(
    app: tauri::AppHandle,
    model: String,
    prompt: String,
    options: Option<LLMOptions>,
) -> Result<(), String> {
    let client = Client::new();

    // Формирование JSON‑тела запроса
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
        // Добавьте обработку других параметров по необходимости
    }
    let request_body = Value::Object(body);

    // Отправка POST‑запроса к Ollama REST API
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // Используем bytes_stream (убедитесь, что в Cargo.toml включена функция "stream")
    let mut stream = response.bytes_stream();

    tauri::async_runtime::spawn(async move {
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk_bytes) => {
                    if let Ok(text) = std::str::from_utf8(chunk_bytes.as_ref()) {
                        // Используем метод emit вместо emit_all для Tauri v2
                        let _ = app.emit("ollama-output", text.to_string());
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка при чтении потока: {:?}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Команда для получения установленных моделей по эндпоинту /api/tags
#[tauri::command]
async fn get_installed_models() -> Result<Vec<String>, String> {
    let client = Client::new();
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // Определяем структуру ответа
    #[derive(Deserialize)]
    struct TagResponse {
        models: Vec<ModelTag>,
    }
    #[derive(Deserialize)]
    struct ModelTag {
        name: String,
        // Остальные поля можно добавить при необходимости
    }

    let tag_response: TagResponse = response
        .json()
        .await
        .map_err(|e| format!("Ошибка парсинга JSON: {}", e))?;

    // Извлекаем имена моделей
    let model_names = tag_response
        .models
        .into_iter()
        .map(|tag| tag.name)
        .collect();
    Ok(model_names)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Если требуется инициализация плагинов, например, tauri-plugin-websocket, подключите их здесь:
        // .plugin(tauri_plugin_websocket::init())
        .invoke_handler(tauri::generate_handler![run_ollama, get_installed_models])
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске Tauri-приложения");
}
