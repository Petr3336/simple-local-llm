use async_trait::async_trait;
use futures::StreamExt;
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaModel, Special},
    sampling::LlamaSampler,
};
use log::{debug, error, info, warn};
use reqwest::Client;
use std::fs;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::model_provider::{LLMOptions, ModelProvider};

pub struct LlamaCppProvider {
    models_dir: PathBuf,
    stop_flag: Arc<AtomicBool>,
    running: Arc<Mutex<bool>>,
}

impl LlamaCppProvider {
    pub async fn new(app: &AppHandle) -> Self {
        let mut app_dir = app.path().cache_dir().expect("Failed to get app data dir");

        #[cfg(not(target_os = "android"))]
        {
            app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
        }

        if let Err(e) = tokio::fs::create_dir_all(&app_dir).await {
            error!("Failed to create app data dir: {:?}", e);
        } else {
            info!("App data dir ensured: {:?}", app_dir);
        }

        let models_dir = app_dir.join("models");

        if let Err(e) = tokio::fs::create_dir_all(&models_dir).await {
            error!("Failed to create models directory: {:?}", e);
        } else {
            info!("Models directory ensured at {:?}", models_dir);
        }

        Self {
            models_dir,
            stop_flag: Arc::new(AtomicBool::new(false)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    fn model_path(&self, name: &str) -> PathBuf {
        self.models_dir.join(name)
    }

    fn ensure_models_dir_exists(&self) -> Result<(), String> {
        if !self.models_dir.exists() {
            fs::create_dir_all(&self.models_dir)
                .map_err(|e| format!("Failed to create models dir: {}", e))?;
        }
        Ok(())
    }
}

#[async_trait]
impl ModelProvider for LlamaCppProvider {
    fn name(&self) -> &'static str {
        "llama.cpp"
    }

    async fn get_installed_models(&self) -> Result<Vec<String>, String> {
        self.ensure_models_dir_exists()?;
        debug!("Scanning models directory for installed models..."); // [log]

        let entries = fs::read_dir(&self.models_dir)
            .map_err(|e| format!("Failed to read models directory: {}", e))?;

        let mut models = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "gguf" {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        debug!("Found model file: {}", name); // [log]
                        models.push(name.to_string());
                    }
                }
            }
        }

        info!("Installed models: {:?}", models); // [log]
        Ok(models)
    }

    async fn run_model(
        &self,
        app: AppHandle,
        model: String,
        prompt: String,
        options: Option<LLMOptions>,
    ) -> Result<(), String> {
        self.ensure_models_dir_exists()?;

        {
            let mut running = self.running.lock().unwrap();
            if *running {
                warn!("Attempted to run model while one is already running"); // [log]
                return Err("Модель уже запущена".to_string());
            }
            *running = true;
        }

        info!("Starting model: {} with prompt: {:?}", model, prompt); // [log]
        self.stop_flag.store(false, Ordering::SeqCst);

        let result = async {
            let backend =
                LlamaBackend::init().map_err(|e| format!("Backend init error: {:?}", e))?;
            info!("Llama backend initialized"); // [log]

            let model_path = self.model_path(&model);
            debug!("Loading model from path: {:?}", model_path); // [log]

            let model_params = Default::default();
            let llama = LlamaModel::load_from_file(&backend, model_path, &model_params)
                .map_err(|e| format!("Failed to load model: {:?}", e))?;
            info!("Model loaded successfully"); // [log]

            let ctx_params = LlamaContextParams::default().with_n_ctx(Some(
                options
                    .and_then(|o| o.num_ctx)
                    .and_then(NonZeroU32::new)
                    .unwrap_or_else(|| NonZeroU32::new(2048).unwrap()),
            ));

            let mut ctx = llama
                .new_context(&backend, ctx_params)
                .map_err(|e| format!("Failed to create context: {:?}", e))?;
            debug!("Context created"); // [log]

            let tokens = llama
                .str_to_token(&prompt, AddBos::Always)
                .map_err(|e| format!("Tokenization failed: {:?}", e))?;
            debug!("Prompt tokenized into {} tokens", tokens.len()); // [log]

            let mut batch = LlamaBatch::new(512, 1);
            for (i, token) in tokens.iter().enumerate() {
                batch
                    .add(*token, i as i32, &[0], i == tokens.len() - 1)
                    .map_err(|e| format!("Batch addition error: {:?}", e))?;
            }

            ctx.decode(&mut batch)
                .map_err(|e| format!("Decoding failed: {:?}", e))?;
            debug!("Initial decoding complete"); // [log]

            let mut sampler = LlamaSampler::greedy();
            let mut n_cur = batch.n_tokens();

            while n_cur < (ctx.n_ctx() as usize).try_into().unwrap() {
                if self.stop_flag.load(Ordering::SeqCst) {
                    info!("Model run stopped by user"); // [log]
                    let _ = app.emit("stop-model", ());
                    break;
                }

                let token = sampler.sample(&ctx, batch.n_tokens() - 1);

                if llama.is_eog_token(token) {
                    info!("End-of-generation token received"); // [log]
                    break;
                }

                if llama
                    .token_to_str(token, Special::Tokenize)
                    .unwrap_or_default()
                    == "<|end_of_text|>"
                {
                    info!("End-of-text token encountered"); // [log]
                    break;
                }

                let output = llama
                    .token_to_str(token, Special::Tokenize)
                    .map_err(|e| format!("Token to string error: {:?}", e))?;

                debug!("Model output token: {}", output); // [log]

                app.emit("model-output", output.to_string())
                    .map_err(|e| format!("Failed to emit event: {:?}", e))?;

                batch.clear();
                batch
                    .add(token, n_cur as i32, &[0], true)
                    .map_err(|e| format!("Batch addition error: {:?}", e))?;

                ctx.decode(&mut batch)
                    .map_err(|e| format!("Decoding failed: {:?}", e))?;

                n_cur += 1;
            }

            Ok(())
        }
        .await;

        let mut running = self.running.lock().unwrap();
        *running = false;

        match &result {
            Ok(_) => info!("Model run completed successfully"), // [log]
            Err(e) => error!("Model run failed: {}", e),        // [log]
        }

        result
    }

    async fn download_model(&self, app: tauri::AppHandle, model: String) -> Result<(), String> {
        info!("Downloading model: {}", model);

        let (repo, filename) = model.split_once(':').ok_or_else(|| {
            let msg = "Model format must be <repo>:<filename.gguf>".to_string();
            error!("{}", msg);
            msg
        })?;

        if !self.models_dir.exists() {
            tokio::fs::create_dir_all(&self.models_dir)
                .await
                .map_err(|e| {
                    let msg = format!("Failed to create models dir: {:?}", e);
                    error!("{}", msg);
                    msg
                })?;
            info!("Created models directory: {:?}", self.models_dir);
        }

        let url = format!("https://huggingface.co/{}/resolve/main/{}", repo, filename);

        debug!("Fetching model from URL: {}", url);

        let client = Client::new();
        let response = client.get(&url).send().await.map_err(|e| {
            let msg = format!("Failed to send GET request: {:?}", e);
            error!("{}", msg);
            msg
        })?;

        if !response.status().is_success() {
            let msg = format!(
                "Failed to download model. HTTP error: {}",
                response.status()
            );
            error!("{}", msg);
            return Err(msg);
        }

        let total_size = response.content_length().ok_or_else(|| {
            let msg = "Response did not include content length.".to_string();
            error!("{}", msg);
            msg
        })?;

        let mut stream = response.bytes_stream();
        let target_path = self.model_path(filename);
        let mut file = File::create(&target_path).await.map_err(|e| {
            let msg = format!("Failed to create file: {:?}", e);
            error!("{}", msg);
            msg
        })?;

        let mut downloaded: u64 = 0;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                let msg = format!("Error reading stream: {:?}", e);
                error!("{}", msg);
                msg
            })?;

            file.write_all(&chunk).await.map_err(|e| {
                let msg = format!("Failed to write chunk to file: {:?}", e);
                error!("{}", msg);
                msg
            })?;

            downloaded += chunk.len() as u64;
            let progress = downloaded as f32 / total_size as f32;
            let _ = app.emit("model-download-progress", progress);
        }

        info!("Model downloaded to {:?}", target_path);
        Ok(())
    }

    async fn delete_model(&self, model: String) -> Result<(), String> {
        let path = self.model_path(&model);
        if path.exists() {
            fs::remove_file(path).map_err(|e| format!("Failed to delete model: {:?}", e))?;
            info!("Model file deleted: {}", model); // [log]
        } else {
            warn!("Attempted to delete non-existent model: {}", model); // [log]
            return Err("Model file does not exist".to_string());
        }
        Ok(())
    }

    async fn stop_model(&self) -> Result<(), String> {
        self.stop_flag.store(true, Ordering::SeqCst);
        info!("Stop flag set for current model run"); // [log]
        Ok(())
    }
}
