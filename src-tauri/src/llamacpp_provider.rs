use async_trait::async_trait;
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaModel, Special},
    sampling::LlamaSampler,
};
use std::fs;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Emitter};

use hf_hub::api::sync::ApiBuilder;

use crate::model_provider::{LLMOptions, ModelProvider};

pub struct LlamaCppProvider {
    models_dir: PathBuf,
    stop_flag: Arc<AtomicBool>,
    running: Arc<Mutex<bool>>,
}

impl LlamaCppProvider {
    pub fn new(app: &AppHandle) -> Self {
        let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
        fs::create_dir_all(&app_dir)?;

        let models_dir = app_dir.join("models");

        if let Err(e) = fs::create_dir_all(&models_dir) {
            eprintln!("Failed to create models directory: {}", e);
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

        let entries = fs::read_dir(&self.models_dir)
            .map_err(|e| format!("Failed to read models directory: {}", e))?;

        let mut models = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "gguf" {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        models.push(name.to_string());
                    }
                }
            }
        }

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
                return Err("Модель уже запущена".to_string());
            }
            *running = true;
        }

        self.stop_flag.store(false, Ordering::SeqCst);

        let result = async {
            let backend = LlamaBackend::init().map_err(|e| format!("Backend init error: {:?}", e))?;
            let model_path = self.model_path(&model);

            let model_params = Default::default();
            let llama = LlamaModel::load_from_file(&backend, model_path, &model_params)
                .map_err(|e| format!("Failed to load model: {:?}", e))?;

            let ctx_params = LlamaContextParams::default().with_n_ctx(
                Some(
                    options
                        .and_then(|o| o.num_ctx)
                        .and_then(NonZeroU32::new)
                        .unwrap_or_else(|| NonZeroU32::new(2048).unwrap()),
                ),
            );

            let mut ctx = llama
                .new_context(&backend, ctx_params)
                .map_err(|e| format!("Failed to create context: {:?}", e))?;

            let tokens = llama
                .str_to_token(&prompt, AddBos::Always)
                .map_err(|e| format!("Tokenization failed: {:?}", e))?;

            let mut batch = LlamaBatch::new(512, 1);
            for (i, token) in tokens.iter().enumerate() {
                batch
                    .add(*token, i as i32, &[0], i == tokens.len() - 1)
                    .map_err(|e| format!("Batch addition error: {:?}", e))?;
            }

            ctx.decode(&mut batch)
                .map_err(|e| format!("Decoding failed: {:?}", e))?;

            let mut sampler = LlamaSampler::greedy();
            let mut n_cur = batch.n_tokens();

            while n_cur < (ctx.n_ctx() as usize).try_into().unwrap() {
                if self.stop_flag.load(Ordering::SeqCst) {
                    let _ = app.emit("stop-model", ());
                    break;
                }

                let token = sampler.sample(&ctx, batch.n_tokens() - 1);

                if llama.is_eog_token(token) {
                    break;
                }
    
                if llama.token_to_str(token, Special::Tokenize).unwrap_or_default() == "<|end_of_text|>" {
                    break;
                }

                let output = llama
                    .token_to_str(token, Special::Tokenize)
                    .map_err(|e| format!("Token to string error: {:?}", e))?;

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

        result
    }

    async fn download_model(&self, model: String) -> Result<(), String> {
        self.ensure_models_dir_exists()?;

        let (repo, filename) = model
            .split_once(':')
            .ok_or("Model format must be <repo>:<filename.gguf>")?;

        let api = ApiBuilder::new()
            .with_progress(true)
            .build()
            .map_err(|e| format!("Failed to create HF API: {:?}", e))?;

        let hf_model = api.model(repo.to_string());
        let downloaded = hf_model
            .get(filename)
            .map_err(|e| format!("Download error: {:?}", e))?;

        let target_path = self.model_path(filename);
        fs::copy(downloaded, target_path)
            .map_err(|e| format!("Copy error: {:?}", e))?;

        Ok(())
    }

    async fn delete_model(&self, model: String) -> Result<(), String> {
        let path = self.model_path(&model);
        if path.exists() {
            fs::remove_file(path)
                .map_err(|e| format!("Failed to delete model: {:?}", e))?;
        } else {
            return Err("Model file does not exist".to_string());
        }
        Ok(())
    }

    async fn stop_model(&self) -> Result<(), String> {
        self.stop_flag.store(true, Ordering::SeqCst);
        Ok(())
    }
}
