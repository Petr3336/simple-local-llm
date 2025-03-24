use async_trait::async_trait;
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaModel, Special},
    sampling::LlamaSampler,
};

use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_fs::FsExt;

use crate::model_provider::{LLMOptions, ModelProvider};

pub struct LlamaCppProvider {
    models_dir: PathBuf,
}

impl LlamaCppProvider {
    pub fn new(app: &AppHandle) -> Self {
        let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
        let models_dir = app_dir.join("models");

        if let Err(e) = std::fs::create_dir_all(&models_dir) {
            eprintln!("Failed to create models directory: {}", e);
        }

        Self { models_dir }
    }
}

#[async_trait]
impl ModelProvider for LlamaCppProvider {
    fn name(&self) -> &'static str {
        "llama.cpp"
    }

    async fn get_installed_models(&self) -> Result<Vec<String>, String> {
        let entries = std::fs::read_dir(&self.models_dir)
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
        let backend = LlamaBackend::init().map_err(|e| format!("Backend init error: {:?}", e))?;
        let model_path = self.models_dir.join(model);

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
}
