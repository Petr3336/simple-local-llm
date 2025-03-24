mod model_provider;
mod ollama_provider;
#[cfg(feature = "llama_cpp")]
mod llamacpp_provider;

use model_provider::{ModelProvider, LLMOptions};
use ollama_provider::OllamaProvider;
use tauri::AppHandle;
use std::sync::Arc;

fn get_providers() -> Vec<Arc<dyn ModelProvider>> {
    let providers: Vec<Arc<dyn ModelProvider>> = vec![Arc::new(OllamaProvider)];

    #[cfg(feature = "llama_cpp")]
    {
        use std::path::PathBuf;
        use llamacpp_provider::LlamaCppProvider;
        providers.push(Arc::new(LlamaCppProvider::new(PathBuf::from("models/llama-model.gguf"))));
    }

    providers
}

#[tauri::command]
async fn get_available_providers() -> Vec<String> {
    get_providers().into_iter().map(|p| p.name().to_string()).collect()
}

#[tauri::command]
async fn get_installed_models(provider_name: String) -> Result<Vec<String>, String> {
    for provider in get_providers() {
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
    for provider in get_providers() {
        if provider.name() == provider_name {
            return provider.run_model(app, model, prompt, options).await;
        }
    }
    Err("Провайдер не найден".into())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_available_providers,
            get_installed_models,
            run_model,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске Tauri-приложения");
}
