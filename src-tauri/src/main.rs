#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod model_provider;
mod ollama_provider;

#[cfg(feature = "llama_cpp")]
mod llamacpp_provider;

use model_provider::{LLMOptions, ModelProvider};
use ollama_provider::OllamaProvider;
use std::sync::Arc;
use tauri::AppHandle;

fn get_providers(app: &AppHandle) -> Vec<Arc<dyn ModelProvider>> {
    let mut providers: Vec<Arc<dyn ModelProvider>> = vec![Arc::new(OllamaProvider)];

    #[cfg(feature = "llama_cpp")]
    {
        use llamacpp_provider::LlamaCppProvider;
        providers.push(Arc::new(LlamaCppProvider::new(app)));
    }

    providers
}

#[tauri::command]
async fn get_available_providers(app: AppHandle) -> Vec<String> {
    get_providers(&app)
        .into_iter()
        .map(|p| p.name().to_string())
        .collect()
}

#[tauri::command]
async fn get_installed_models(app: AppHandle, provider_name: String) -> Result<Vec<String>, String> {
    for provider in get_providers(&app) {
        if provider.name() == provider_name {
            return provider.get_installed_models().await;
        }
    }
    Err("Провайдер не найден".into())
}

#[tauri::command]
async fn run_model(
    app: AppHandle,
    provider_name: String,
    model: String,
    prompt: String,
    options: Option<LLMOptions>,
) -> Result<(), String> {
    for provider in get_providers(&app) {
        if provider.name() == provider_name {
            return provider.run_model(app, model, prompt, options).await;
        }
    }
    Err("Провайдер не найден".into())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_available_providers,
            get_installed_models,
            run_model,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске Tauri-приложения");
}
