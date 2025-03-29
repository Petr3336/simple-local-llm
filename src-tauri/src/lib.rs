#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod model_provider;

#[cfg(feature = "ollama")]
mod ollama_provider;

#[cfg(feature = "llama_cpp")]
mod llamacpp_provider;

use log::LevelFilter;
use log::{info, debug, warn, error}; // [log]

use model_provider::{LLMOptions, ModelProvider};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tauri_plugin_log::{Builder as LogBuilder, TargetKind};

use once_cell::sync::OnceCell;

static PROVIDERS: OnceCell<Mutex<Vec<Arc<dyn ModelProvider>>>> = OnceCell::new();


async fn init_providers(app: &AppHandle) {
    let mut providers: Vec<Arc<dyn ModelProvider>> = vec![];

    #[cfg(feature = "ollama")]
    {
        use ollama_provider::OllamaProvider;
        providers.push(Arc::new(OllamaProvider::new()));
    }

    #[cfg(feature = "llama_cpp")]
    {
        use llamacpp_provider::LlamaCppProvider;
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
    debug!("Request to get installed models for provider: {}", provider_name); // [log]
    for provider in get_providers() {
        if provider.name() == provider_name {
            return provider.get_installed_models().await;
        }
    }
    warn!("Provider not found: {}", provider_name); // [log]
    Err("Provider not found".into())
}

#[tauri::command]
async fn run_model(
    app: AppHandle,
    provider_name: String,
    model: String,
    prompt: String,
    options: Option<LLMOptions>,
) -> Result<(), String> {
    info!("Request to run model '{}' on provider '{}'", model, provider_name); // [log]
    for provider in get_providers() {
        if provider.name() == provider_name {
            return provider.run_model(app, model, prompt, options).await;
        }
    }
    warn!("Provider not found: {}", provider_name); // [log]
    Err("Provider not found".into())
}

#[tauri::command]
async fn download_model(app: AppHandle, provider_name: String, model: String) -> Result<(), String> {
    info!("Request to download model '{}' from provider '{}'", model, provider_name); // [log]
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
    info!("Request to delete model '{}' from provider '{}'", model, provider_name); // [log]
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
        .setup(|app| {
            info!("Running setup hook");
            // 2. Получаем AppHandle как отдельную переменную (копия)
            let handle = app.handle().clone();
        
            // 3. Передаём его внутрь async
            tauri::async_runtime::spawn(async move {
                init_providers(&handle).await;
            });

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
        .invoke_handler(tauri::generate_handler![
            get_available_providers,
            get_installed_models,
            run_model,
            download_model,
            delete_model,
            stop_model,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске Tauri-приложения");
}