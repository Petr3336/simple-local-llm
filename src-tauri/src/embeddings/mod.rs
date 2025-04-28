use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::num::NonZeroU32;
use anyhow::{Context, Result};
use log::info;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use serde::Deserialize;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::model::params::LlamaModelParams;
use bytemuck;
use tauri::async_runtime::spawn_blocking;

/// Источник ввода: либо файл, либо строковой контент
#[derive(Deserialize)]
pub enum InputSource {
    File(PathBuf),
    Content(String),
}

/// Tauri-команда для получения эмбеддингов с автоматическим кэшем
///
/// # Аргументы
/// - `app`: дескриптор приложения для получения cache_dir
/// - `model_path`: путь до файла модели
/// - `input`: источник ввода (файл или строка)
///
/// # Возвращает
/// Вектор f32 с эмбеддингами или ошибку в виде строки
#[tauri::command]
pub async fn embed_with_cache(
    app: AppHandle,
    model_path: String,
    input: InputSource,
) -> Result<Vec<f32>, String> {
    let model_path = PathBuf::from(model_path);
    let key = compute_cache_key(&model_path, &input)
        .map_err(|e| e.to_string())?;

    let mut base = app.path().cache_dir().map_err(|e| e.to_string())?;

    #[cfg(not(target_os = "android"))]
    {
        base = app
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");
    }

    let cache_dir = base.join("embeddings");
    let cache_file = cache_dir.join(format!("{}.bin", key));
    info!("embedding cache file: {}", cache_file.as_path().to_str().unwrap());

    if cache_file.exists() {
        let raw = fs::read(&cache_file)
            .map_err(|e| e.to_string())?;
        let floats: &[f32] = bytemuck::cast_slice(&raw);
        return Ok(floats.to_vec());
    }

    let embedding = compute_embedding(&model_path, &input)
        .await
        .map_err(|e| e.to_string())?;
    let bytes = bytemuck::cast_slice(&embedding);

    fs::create_dir_all(&cache_dir)
        .map_err(|e| e.to_string())?;
    fs::write(&cache_file, bytes)
        .map_err(|e| e.to_string())?;

    Ok(embedding)
}

async fn compute_embedding(model_path: &Path, input: &InputSource) -> Result<Vec<f32>> {
    // Инициализация Llama
    let backend = LlamaBackend::init().context("Failed to init LlamaBackend")?;
    let model_params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
        .context("Failed to load LlamaModel")?;

    // Считываем входной текст
    let prompt_str = match input {
        InputSource::File(p) => fs::read_to_string(p)
            .with_context(|| format!("Failed to read input file {:?}", p))?,
        InputSource::Content(s) => s.clone(),
    };

    // Токенизируем перед созданием контекста
    let tokens = model
        .str_to_token(&prompt_str, AddBos::Always)
        .context("Tokenization failed")?;
    let token_count = tokens.len() as u32;

    // Указываем размер контекста 4096 и n_ubatch = число токенов
    let ctx_params = LlamaContextParams::default()
        .with_embeddings(true)
        .with_n_ctx(Some(NonZeroU32::new(4096).unwrap()))
        .with_n_ubatch(token_count);
    let mut ctx = model
        .new_context(&backend, ctx_params)
        .context("Failed to create LlamaContext")?;

    let n_ctx = ctx.n_ctx() as usize;
    if tokens.len() > n_ctx {
        anyhow::bail!(
            "Prompt too long: {} tokens, but context window is {}",
            tokens.len(),
            n_ctx
        );
    }

    // Создаём батч и кодируем
    let mut batch = LlamaBatch::new(n_ctx, 1);
    batch.add_sequence(&tokens, 0, false)
        .context("Failed to add sequence to batch")?;

    ctx.clear_kv_cache();
    ctx.decode(&mut batch)
        .context("Decoding failed")?;

    let embedding = ctx
        .embeddings_seq_ith(0)
        .context("Failed to extract embeddings")?;
    Ok(embedding.to_vec())
}

fn compute_cache_key(model_path: &Path, input: &InputSource) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(model_path.to_string_lossy().as_bytes());
    match input {
        InputSource::File(p) => {
            let data = fs::read(p)
                .with_context(|| format!("Failed to read file for hashing: {:?}", p))?;
            hasher.update(&data);
        }
        InputSource::Content(s) => {
            hasher.update(s.as_bytes());
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

// 1) Синхронная функция, вычисляющая эмбеддинг (с учётом кэша)
pub fn embed_sync(
    app: &AppHandle,
    backend: &LlamaBackend,
    model: &LlamaModel,
    model_path: &Path,
    cache_root: &Path,
    input: InputSource,
) -> Result<Vec<f32>, String> {
    // 1) Ключ кэша
    let key = compute_cache_key(model_path, &input)
        .map_err(|e| e.to_string())?;
    let cache_file = cache_root.join(format!("{}.bin", key));

    // 2) Если уже есть файл — читаем и возвращаем
    if cache_file.exists() {
        let raw = fs::read(&cache_file)
            .map_err(|e| e.to_string())?;
        let floats: &[f32] = bytemuck::cast_slice(&raw);
        return Ok(floats.to_vec());
    }

    // 3) Считать вход
    let prompt_str = match &input {
        InputSource::File(path) => fs::read_to_string(path)
            .map_err(|e| e.to_string())?,
        InputSource::Content(s) => s.clone(),
    };

    // 4) Токенизировать
    let tokens = model
        .str_to_token(&prompt_str, AddBos::Always)
        .map_err(|e| e.to_string())?;
    let token_count = tokens.len() as u32;

    // 5) Создать контекст с эмбеддингами
    let ctx_params = LlamaContextParams::default()
        .with_embeddings(true)
        .with_n_ctx(Some(NonZeroU32::new(4096).unwrap()))
        .with_n_ubatch(token_count);
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| e.to_string())?;

    // 6) Проверка на окно контекста
    if tokens.len() > ctx.n_ctx() as usize {
        return Err(format!(
            "Prompt too long: {} tokens, but context window is {}",
            tokens.len(),
            ctx.n_ctx()
        ));
    }

    // 7) Декодинг и извлечение эмбеддинга
    let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
    batch
        .add_sequence(&tokens, 0, false)
        .map_err(|e| e.to_string())?;
    ctx.clear_kv_cache();
    ctx.decode(&mut batch)
        .map_err(|e| e.to_string())?;
    let embedding = ctx
        .embeddings_seq_ith(0)
        .map_err(|e| e.to_string())?
        .to_vec();

    // 8) Сохранить в кэш
    fs::create_dir_all(cache_root)
        .map_err(|e| e.to_string())?;
    let bytes = bytemuck::cast_slice(&embedding);
    fs::write(&cache_file, bytes)
        .map_err(|e| e.to_string())?;

    Ok(embedding)
}

#[tauri::command]
pub async fn retrieve_context(
    app: AppHandle,
    model_path: String,
    query_text: String,
    file_paths: Vec<String>,
    segment_size: u32,
    top_n: usize,                  // ← новый параметр
) -> Result<String, String> {
    // копируем top_n внутрь blocking-блока
    let n = top_n;
    let result_str = spawn_blocking(move || {
        // a) Инициализация бэкенда и модели
        let backend = LlamaBackend::init().map_err(|e| e.to_string())?;
        let model = LlamaModel::load_from_file(
            &backend,
            &model_path,
            &LlamaModelParams::default(),
        )
        .map_err(|e| e.to_string())?;

        // b) Папка кэша
        let base = app.path().cache_dir().map_err(|e| e.to_string())?;
        let cache_root = base.join("embeddings");

        // c) Эмбеддинг запроса
        let query_emb = embed_sync(
            &app,
            &backend,
            &model,
            Path::new(&model_path),
            &cache_root,
            InputSource::Content(query_text.clone()),
        )?;

        // d) Сбор всех сегментов с их эмбеддингами
        let mut segments: Vec<(String, Vec<f32>)> = Vec::new();
        for path in &file_paths {
            let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let tokens = model
                .str_to_token(&text, AddBos::Always)
                .map_err(|e| e.to_string())?;
            for chunk in tokens.chunks(segment_size as usize) {
                let text_seg = model
                    .tokens_to_str(chunk, Special::Plaintext)
                    .map_err(|e| e.to_string())?;
                let emb = embed_sync(
                    &app,
                    &backend,
                    &model,
                    Path::new(&model_path),
                    &cache_root,
                    InputSource::Content(text_seg.clone()),
                )?;
                segments.push((text_seg, emb));
            }
        }

        // e) Функция для косинусной схожести
        fn cosine(a: &[f32], b: &[f32]) -> f32 {
            let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
            let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let nb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
            if na == 0.0 || nb == 0.0 {
                0.0
            } else {
                dot / (na * nb)
            }
        }

        // f) Сортировка по убыванию схожести
        segments.sort_by(|(_, emb_a), (_, emb_b)| {
            let sim_a = cosine(&query_emb, emb_a);
            let sim_b = cosine(&query_emb, emb_b);
            sim_b.partial_cmp(&sim_a).unwrap_or(Ordering::Equal)
        });

        // g) Берём только n верхних и собираем в одну строку
        let top_segments = segments
            .into_iter()
            .take(n)
            .map(|(text, _)| text)
            .collect::<Vec<_>>();

        Ok::<String, String>(top_segments.join("\n\n"))
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(result_str)
}