use std::path::PathBuf;

use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::{AddBos, Special};

#[tauri::command]
pub async fn generate_embeddings(prompt: String, model_path: String) -> Result<Vec<f32>, String> {
    async move {
        let backend = LlamaBackend::init()?;

        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, PathBuf::from(model_path), &model_params)?;

        let ctx_params = LlamaContextParams::default()
            .with_embeddings(true);

        let mut ctx = model.new_context(&backend, ctx_params)?;

        let tokens = model.str_to_token(&prompt, AddBos::Always)?;

        let n_ctx = ctx.n_ctx() as usize;
        if tokens.len() > n_ctx {
            anyhow::bail!("Prompt too long for context window");
        }

        let mut batch = LlamaBatch::new(n_ctx, 1);
        batch.add_sequence(&tokens, 0, false)?;

        ctx.clear_kv_cache();
        ctx.decode(&mut batch)?;

        let embedding = ctx.embeddings_seq_ith(0)?;

        Ok(embedding.to_vec())
    }
    .await
    .map_err(|e| e.to_string())
}
