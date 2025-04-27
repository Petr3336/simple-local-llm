use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaModel, AddBos};

use bytemuck;

#[tauri::command]
pub async fn prepare_embeddings(
    app: AppHandle,
    original_model_path: String,
    embedding_filename: String,
) -> Result<String, String> {
    // Загружаем модель напрямую без копирования
    let model_path = PathBuf::from(original_model_path);

    let backend = LlamaBackend::init().map_err(|e| e.to_string())?;
    let model_params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
        .map_err(|e| e.to_string())?;

    let ctx_params = LlamaContextParams::default()
        .with_embeddings(true);

    let mut ctx = model.new_context(&backend, ctx_params)
        .map_err(|e| e.to_string())?;

    let tokens = model.str_to_token("", AddBos::Always)
        .map_err(|e| e.to_string())?;

    let n_ctx = ctx.n_ctx() as usize;

    if tokens.len() > n_ctx {
        return Err("Prompt too long for context window".to_string());
    }

    let mut batch = LlamaBatch::new(n_ctx, 1);
    batch.add_sequence(&tokens, 0, false)
        .map_err(|e| e.to_string())?;

    ctx.clear_kv_cache();
    ctx.decode(&mut batch)
        .map_err(|e| e.to_string())?;

    let embedding = ctx.embeddings_seq_ith(0)
        .map_err(|e| e.to_string())?;

    let embedding_path = save_embedding_to_cache(&app, &embedding, &embedding_filename)?;

    Ok(embedding_path.to_string_lossy().into_owned())
}

fn save_embedding_to_cache(app: &AppHandle, embedding: &[f32], filename: &str) -> Result<PathBuf, String> {
    let cache_path = app
        .path()
        .cache_dir()
        .map_err(|e| e.to_string())?
        .join("simple-local-llm")
        .join("embeddings");

    fs::create_dir_all(&cache_path)
        .map_err(|e| e.to_string())?;

    let dest_path = cache_path.join(filename);

    let bytes = bytemuck::cast_slice(embedding);
    fs::write(&dest_path, bytes)
        .map_err(|e| e.to_string())?;

    Ok(dest_path)
}
