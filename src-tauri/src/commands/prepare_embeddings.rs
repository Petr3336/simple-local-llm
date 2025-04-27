use std::fs;
use std::path::{PathBuf};
use tauri::api::path::cache_dir;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::AddBos;
use bytemuck;
use anyhow::Result;

#[tauri::command]
pub async fn prepare_embeddings(
    original_model_path: String,
    embedding_filename: String,
) -> Result<String, String> {
    async move {
        // 1. Копируем модель в кэш
        let cached_model_path = copy_model_to_cache(&original_model_path)?;

        // 2. Инициализация
        let backend = LlamaBackend::init()?;
        let model_params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, cached_model_path.clone(), &model_params)?;

        let ctx_params = LlamaContextParams::default()
            .with_embeddings(true);

        let mut ctx = model.new_context(&backend, ctx_params)?;

        // 3. Генерируем эмбеддинг для пустого текста
        let tokens = model.str_to_token("", AddBos::Always)?;
        let n_ctx = ctx.n_ctx() as usize;
        if tokens.len() > n_ctx {
            anyhow::bail!("Prompt too long for context window");
        }

        let mut batch = LlamaBatch::new(n_ctx, 1);
        batch.add_sequence(&tokens, 0, false)?;

        ctx.clear_kv_cache();
        ctx.decode(&mut batch)?;

        let embedding = ctx.embeddings_seq_ith(0)?;

        // 4. Сохраняем эмбеддинг в кэш
        let embedding_path = save_embedding_to_cache(&embedding, &embedding_filename)?;

        Ok(embedding_path.to_string_lossy().into_owned())
    }
    .await
    .map_err(|e| e.to_string())
}

fn copy_model_to_cache(original_path: &str) -> Result<PathBuf, String> {
    let cache_path = cache_dir()
        .ok_or("Cannot find cache dir")?
        .join("simple-local-llm")
        .join("models");

    fs::create_dir_all(&cache_path).map_err(|e| e.to_string())?;

    let filename = PathBuf::from(original_path)
        .file_name()
        .ok_or("Invalid original file name")?;

    let dest_path = cache_path.join(filename);

    fs::copy(original_path, &dest_path).map_err(|e| e.to_string())?;

    Ok(dest_path)
}

fn save_embedding_to_cache(embedding: &[f32], filename: &str) -> Result<PathBuf, String> {
    let cache_path = cache_dir()
        .ok_or("Cannot find cache dir")?
        .join("simple-local-llm")
        .join("embeddings");

    fs::create_dir_all(&cache_path).map_err(|e| e.to_string())?;

    let dest_path = cache_path.join(filename);

    let bytes = bytemuck::cast_slice(embedding);
    fs::write(&dest_path, bytes).map_err(|e| e.to_string())?;

    Ok(dest_path)
}
