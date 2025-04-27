mod model_provider;
pub mod model_providers;

pub mod function_provider;
use function_provider::LlmFunction;
pub mod function_providers;

mod commands {
    pub mod embeddings;
}

use log::LevelFilter;
use log::{debug, error, info, warn}; // [log]

use model_provider::{LLMOptions, ModelProvider};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri_plugin_log::{Builder as LogBuilder, TargetKind};

use once_cell::sync::OnceCell;

static PROVIDERS: OnceCell<Mutex<Vec<Arc<dyn ModelProvider>>>> = OnceCell::new();
//static FUNCTIONS: OnceCell<Vec<Arc<dyn LlmFunction>>> = OnceCell::new();

async fn init_providers(app: &AppHandle) {
    let mut providers: Vec<Arc<dyn ModelProvider>> = vec![];

    #[cfg(feature = "ollama")]
    {
        use crate::model_providers::ollama_provider::OllamaProvider;
        providers.push(Arc::new(OllamaProvider::new()));
    }

    #[cfg(feature = "llama_cpp")]
    {
        use crate::model_providers::llamacpp_provider::LlamaCppProvider;
        let provider = LlamaCppProvider::new(app).await;
        providers.push(Arc::new(provider));
    }

    let count = providers.len();
    PROVIDERS.set(Mutex::new(providers)).ok();
    info!("Initialized {} model provider(s)", count);
}

fn get_providers() -> Vec<Arc<dyn ModelProvider>> {
    PROVIDERS
        .get()
        .expect("Providers not initialized")
        .lock()
        .unwrap()
        .clone()
}

/* pub fn initialize_functions() -> HashMap<String, Arc<dyn LlmFunction>> {
    let functions = function_providers::all_functions();

    functions
        .into_iter()
        .map(|f| {
            let def = f.definition();
            (def.name.clone(), f)
        })
        .collect()
} */

pub fn initialize_functions() -> Vec<Arc<dyn LlmFunction>> {
    function_providers::all_functions()
}

#[tauri::command]
async fn get_available_providers() -> Vec<String> {
    let names: Vec<_> = get_providers()
        .into_iter()
        .map(|p| p.name().to_string())
        .collect();
    debug!("Available providers: {:?}", names); // [log]
    names
}

#[tauri::command]
async fn get_installed_models(provider_name: String) -> Result<Vec<String>, String> {
    debug!(
        "Request to get installed models for provider: {}",
        provider_name
    ); // [log]
    for provider in get_providers() {
        if provider.name() == provider_name {
            return provider.get_installed_models().await;
        }
    }
    warn!("Provider not found: {}", provider_name); // [log]
    Err("Provider not found".into())
}

use crate::function_provider::FunctionDefinition;
#[tauri::command]
async fn get_available_functions() -> Result<Vec<FunctionDefinition>, String> {
    info!("Request to get available functions"); // [log]

    let available_functions = function_providers::all_functions()
        .into_iter()
        .map(|func| func.definition())
        .collect();

    Ok(available_functions)
}

#[tauri::command]
async fn run_model(
    app: AppHandle,
    provider_name: String,
    model: String,
    messages: Vec<serde_json::Value>,
    options: Option<LLMOptions>,
    chat_id: String,
) -> Result<(), String> {
    info!(
        "Request to run model '{}' on provider '{}'",
        model, provider_name
    ); // [log]
    for provider in get_providers() {
        if provider.name() == provider_name {
            return provider
                .run_model(app, model, messages, options, chat_id)
                .await;
        }
    }
    warn!("Provider not found: {}", provider_name); // [log]
    Err("Provider not found".into())
}

#[tauri::command]
async fn download_model(
    app: AppHandle,
    provider_name: String,
    model: String,
) -> Result<(), String> {
    info!(
        "Request to download model '{}' from provider '{}'",
        model, provider_name
    ); // [log]
    for provider in get_providers() {
        if provider.name() == provider_name {
            return provider.download_model(app, model).await;
        }
    }
    warn!("Provider not found: {}", provider_name); // [log]
    Err("Provider not found".into())
}

#[tauri::command]
async fn delete_model(provider_name: String, model: String) -> Result<(), String> {
    info!(
        "Request to delete model '{}' from provider '{}'",
        model, provider_name
    ); // [log]
    for provider in get_providers() {
        if provider.name() == provider_name {
            return provider.delete_model(model).await;
        }
    }
    warn!("Provider not found: {}", provider_name); // [log]
    Err("Provider not found".into())
}

#[tauri::command]
async fn stop_model(provider_name: String) -> Result<(), String> {
    info!("Request to stop model on provider '{}'", provider_name); // [log]
    for provider in get_providers() {
        if provider.name() == provider_name {
            let _ = provider.stop_model().await;
            return Ok(());
        }
    }
    warn!("Provider not found: {}", provider_name); // [log]
    Err("Provider not found".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            info!("Running setup hook");
            // 2. Получаем AppHandle как отдельную переменную (копия)
            let handle = app.handle().clone();

            // 3. Передаём его внутрь async
            tauri::async_runtime::spawn(async move {
                init_providers(&handle).await;
            });

            /* tauri::async_runtime::spawn(async move {
                // 👇 Инициализация функций
                let functions = initialize_functions();
                FUNCTIONS.set(functions).ok();
            }); */

            Ok(())
        })
        .plugin(
            LogBuilder::default()
                .level(log_level)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    }),
                    tauri_plugin_log::Target::new(TargetKind::Stdout),
                    tauri_plugin_log::Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_pinia::init())
        .invoke_handler(tauri::generate_handler![
            get_available_providers,
            get_installed_models,
            get_available_functions,
            run_model,
            download_model,
            delete_model,
            stop_model,
            commands::embeddings::generate_embeddings,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске Tauri-приложения");
}
