use async_trait::async_trait;
use futures::StreamExt;
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaChatMessage, LlamaChatTemplate, LlamaModel, Special},
    sampling::LlamaSampler,
};
use log::{debug, error, info, warn};
use reqwest::Client;
use serde_json::json;
use std::{fs, ptr::null};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use minijinja::{Environment, context, Value};


use crate::model_provider::{LLMOptions, ModelProvider};
use crate::initialize_functions;

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
        messages: Vec<serde_json::Value>,
        options: Option<LLMOptions>,
        chat_id: String,
    ) -> Result<(), String> {
        self.ensure_models_dir_exists()?;

        {
            let mut running = self.running.lock().unwrap();
            if *running {
                warn!("Attempted to run model while one is already running");
                return Err("Модель уже запущена".to_string());
            }
            *running = true;
        }

        info!("Starting model: {} with messages: {:?}", model, messages);
        self.stop_flag.store(false, Ordering::SeqCst);

        let model_path = self.model_path(&model);
        let stop_flag = self.stop_flag.clone();
        let options = options.clone();
        let chat_id = chat_id.clone();
        let messages = messages.clone();

        let result = tokio::task::spawn_blocking(move || {
            let backend =
                LlamaBackend::init().map_err(|e| format!("Backend init error: {:?}", e))?;
            info!("Llama backend initialized");

            debug!("Loading model from path: {:?}", model_path);

            let model_params = Default::default();
            let llama = LlamaModel::load_from_file(&backend, model_path, &model_params)
                .map_err(|e| format!("Failed to load model: {:?}", e))?;
            info!("Model loaded successfully");
            let ctx_params = LlamaContextParams::default().with_n_ctx(Some(
                options.clone()
                    .and_then(|o| o.num_ctx)
                    .and_then(NonZeroU32::new)
                    .unwrap_or_else(|| NonZeroU32::new(2048).unwrap()),
            ));

            let mut ctx = llama
                .new_context(&backend, ctx_params)
                .map_err(|e| format!("Failed to create context: {:?}", e))?;
            debug!("Context created");

            // Формируем промпт из сообщений
            let mut prompt = String::new();
            for msg in messages.iter() {
                if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        prompt.push_str(&format!("{}: {}\n", role, content));
                    }
                }
            }
            prompt.push_str("assistant: ");

            let tokens = llama
                .str_to_token(&prompt, AddBos::Always)
                .map_err(|e| format!("Tokenization failed: {:?}", e))?;
            debug!("Prompt tokenized into {} tokens", tokens.len());

            let mut batch = LlamaBatch::new(512, 1);
            for (i, token) in tokens.iter().enumerate() {
                batch
                    .add(*token, i as i32, &[0], i == tokens.len() - 1)
                    .map_err(|e| format!("Batch addition error: {:?}", e))?;
            }

            ctx.decode(&mut batch)
                .map_err(|e| format!("Decoding failed: {:?}", e))?;
            debug!("Initial decoding complete");

            let mut sampler = LlamaSampler::greedy();
            let mut n_cur = batch.n_tokens();
            let mut full_response = String::new();

            while n_cur < (ctx.n_ctx() as usize).try_into().unwrap() {
                if stop_flag.load(Ordering::SeqCst) {
                    info!("Model run stopped by user");
                    let _ = app.emit("stop-model", ());
                    break;
                }

                let token = sampler.sample(&ctx, batch.n_tokens() - 1);

                if llama.is_eog_token(token) {
                    info!("End-of-generation token received");
                    break;
                }

                if llama
                    .token_to_str(token, Special::Tokenize)
                    .unwrap_or_default()
                    == "<|end_of_text|>"
                {
                    info!("End-of-text token encountered");
                    break;
                }

                let output = llama
                    .token_to_str(token, Special::Tokenize)
                    .map_err(|e| format!("Token to string error: {:?}", e))?;

                debug!("Model output token: {}", output);
                full_response.push_str(&output);

                // Если включен стриминг, отправляем каждый токен
                if options.as_ref().map_or(false, |opts| opts.stream) {
                    let payload = json!({
                        "chat_id": chat_id,
                        "output": json!({
                            "model": model,
                            "created_at": chrono::Utc::now().to_rfc3339(),
                            "message": {
                                "role": "assistant",
                                "content": output
                            },
                            "done": false
                        }).to_string() + "\n"
                    });
                    let _ = app.emit("model-stream-output", payload.to_string());
                }

                batch.clear();
                batch
                    .add(token, n_cur as i32, &[0], true)
                    .map_err(|e| format!("Batch addition error: {:?}", e))?;

                ctx.decode(&mut batch)
                    .map_err(|e| format!("Decoding failed: {:?}", e))?;

                n_cur += 1;
            }

            // Если стриминг отключен, отправляем полный ответ
            if !options.as_ref().map_or(false, |opts| opts.stream) {
                let output_value = match serde_json::from_str::<serde_json::Value>(&full_response) {
                    Ok(parsed) => parsed,
                    Err(_) => json!(full_response),
                };

                // Проверяем, есть ли вызов функции в ответе
                if let Some(tool_calls) = output_value.get("tool_calls").and_then(|v| v.as_array()) {
                    if let Some(first_tool_call) = tool_calls.first() {
                        if let Some(function) = first_tool_call.get("function") {
                            if let (Some(name), Some(arguments)) = (
                                function.get("name").and_then(|v| v.as_str()),
                                function.get("arguments").cloned(),
                            ) {
                                info!("Received function call: {}", name);
                                let all_functions = initialize_functions();
                                if let Some(func) = all_functions.iter().find(|f| f.definition().name == name) {
                                    let tool_result = futures::executor::block_on(func.call(arguments));
                                    // Отправляем результат вызова функции
                                    let tool_response = json!({
                                        "chat_id": chat_id,
                                        "output": {
                                            "role": "tool",
                                            "content": format!("Результат выполнения функции {name}: {}", tool_result.unwrap_or_default()),
                                            "tool_call_id": name
                                        }
                                    });
                                    let _ = app.emit("model-output", tool_response.to_string());
                                }
                            }
                        }
                    } else {
                        // Если нет вызова функции, отправляем обычный ответ
                        let payload = json!({
                            "chat_id": chat_id,
                            "output": {
                                "model": model,
                                "created_at": chrono::Utc::now().to_rfc3339(),
                                "message": {
                                    "role": "assistant",
                                    "content": full_response
                                },
                                "done": true
                            }
                        });
                        let _ = app.emit("model-output", payload.to_string());
                    }
                } else {
                    // Если нет вызова функции, отправляем обычный ответ
                    let payload = json!({
                        "chat_id": chat_id,
                        "output": {
                            "model": model,
                            "created_at": chrono::Utc::now().to_rfc3339(),
                            "message": {
                                "role": "assistant",
                                "content": full_response
                            },
                            "done": true
                        }
                    });
                    let _ = app.emit("model-output", payload.to_string());
                }
            }

            Ok(())
        })
        .await
        .map_err(|e| format!("Failed to run model: {:?}", e))?;

        let mut running = self.running.lock().unwrap();
        *running = false;

        match &result {
            Ok(_) => info!("Model run completed successfully"),
            Err(e) => error!("Model run failed: {}", e),
        }

        result
    }

    /*     async fn run_model(
        &self,
        app: AppHandle,
        model: String,
        messages: Vec<serde_json::Value>,
        options: Option<LLMOptions>,
        chat_id: String,
    ) -> Result<(), String> {
        self.ensure_models_dir_exists()?;
    
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                warn!("Attempted to run model while one is already running");
                return Err("Модель уже запущена".to_string());
            }
            *running = true;
        }
    
        info!("Starting model: {} with messages: {:?}", model, messages);
        self.stop_flag.store(false, Ordering::SeqCst);
    
        let model_path = self.model_path(&model);
        let stop_flag = self.stop_flag.clone();
        let options = options.clone();
        let chat_id = chat_id.clone();
        let messages = messages.clone();
        
        debug!("Using tools: {:?}", options.clone().unwrap().functions);
    
        let result = tokio::task::spawn_blocking(move || {
            // Инициализация backend
            let backend = LlamaBackend::init()
                .map_err(|e| format!("Backend init error: {:?}", e))?;
            info!("Llama backend initialized");
    
            debug!("Loading model from path: {:?}", model_path);
            let model_params = Default::default();
            let llama = LlamaModel::load_from_file(&backend, model_path, &model_params)
                .map_err(|e| format!("Failed to load model: {:?}", e))?;
            info!("Model loaded successfully");
    
            let ctx_params = LlamaContextParams::default().with_n_ctx(Some(
                options.clone()
                    .and_then(|o| o.num_ctx)
                    .and_then(NonZeroU32::new)
                    .unwrap_or_else(|| NonZeroU32::new(2048).unwrap()),
            ));
            let mut ctx = llama
                .new_context(&backend, ctx_params)
                .map_err(|e| format!("Failed to create context: {:?}", e))?;
            debug!("Context created");
    
            // Получаем оригинальный шаблон
            let original_chat_template = llama.get_chat_template()
                .map_err(|e| format!("Ошибка получения шаблона: {:?}", e))?;
            let template_str = original_chat_template.as_c_str()
                .to_str()
                .map_err(|e| format!("Ошибка конвертации шаблона в UTF-8: {:?}", e))?;
    
            let all_functions = initialize_functions();
            let allowed_function_names = options
                .as_ref()
                .and_then(|opts| opts.functions.clone())
                .unwrap_or_default();
    
            let mut functions_json = vec![];
            for name in &allowed_function_names {
                if let Some(func) = all_functions.get(name) {
                    let def = func.definition();
                    let json = serde_json::json!({
                        "name": def.name,
                        "description": def.description.unwrap_or_default(),
                        "parameters": def.parameters.iter().map(|(key, param)| {
                            (key.clone(), serde_json::json!({
                                "name": param.name,
                                "description": param.description,
                                "type": param.param_type,
                            }))
                        }).collect::<serde_json::Value>()
                    });
                    functions_json.push(json);
                } else {
                    debug!("Функция '{}' не найдена в all_functions", name);
                }
            }
    
            let mut chat_messages = Vec::new();
            let system_message = "You are helpful AI assistant. Don't put a function call inside triple quotes ```";
    
            chat_messages.push(serde_json::json!({
                "role": "system",
                "content": system_message
            }));
    
            for msg in messages.iter() {
                if let (Some(role_str), Some(content)) = (
                    msg.get("role").and_then(|r| r.as_str()),
                    msg.get("content").and_then(|c| c.as_str()),
                ) {
                    chat_messages.push(serde_json::json!({
                        "role": role_str,
                        "content": content
                    }));
                }
            }
    
            let mut env = Environment::new();
            env.add_function("strftime_now", |format: &str| {
                chrono::Utc::now().format(format).to_string()
            });
            env.add_filter("tojson", |value: Value, _args: Value| {
                serde_json::to_string_pretty(&value).map_err(|e| minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    format!("JSON serialization error: {}", e)
                ))
            });    

            env.add_filter("tojson", |value: Value| {
                serde_json::to_string_pretty(&value).map_err(|e| minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    format!("JSON serialization error: {}", e)
                ))
            });
    
            let tmpl = env.template_from_str(template_str)
                .map_err(|e| format!("Ошибка компиляции шаблона MiniJinja: {:?}", e))?;
    
            let template = context! {
                tools => functions_json,
                tools_in_user_message => false,
                messages => chat_messages,
                add_generation_prompt => true,
            };
    
            let rendered_template = tmpl.render(template)
                .map_err(|e| format!("Ошибка рендеринга MiniJinja: {:?}", e))?;
    
            let prompt = rendered_template;
    
            debug!("Generated prompt: {}", prompt);
    
            let tokens = llama
                .str_to_token(&prompt, AddBos::Always)
                .map_err(|e| format!("Tokenization failed: {:?}", e))?;
            debug!("Prompt tokenized into {} tokens", tokens.len());
    
            let mut batch = LlamaBatch::new(512, 1);
            for (i, token) in tokens.iter().enumerate() {
                batch
                    .add(*token, i as i32, &[0], i == tokens.len() - 1)
                    .map_err(|e| format!("Batch addition error: {:?}", e))?;
            }
    
            ctx.decode(&mut batch)
                .map_err(|e| format!("Decoding failed: {:?}", e))?;
            debug!("Initial decoding complete");
    
            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::greedy(),
                LlamaSampler::temp(0.3),
            ]);
            let mut n_cur = batch.n_tokens();
            let mut full_response = String::new();
    
            while n_cur < (ctx.n_ctx() as usize).try_into().unwrap() {
                if stop_flag.load(Ordering::SeqCst) {
                    info!("Model run stopped by user");
                    let _ = app.emit("stop-model", ());
                    break;
                }
    
                let token = sampler.sample(&ctx, batch.n_tokens() - 1);
    
                if llama.is_eog_token(token) {
                    info!("End-of-generation token received");
                    break;
                }
    
                if llama
                    .token_to_str(token, Special::Tokenize)
                    .unwrap_or_default() == "<|end_of_text|>"
                {
                    info!("End-of-text token encountered");
                    break;
                }
    
                let output = llama
                    .token_to_str(token, Special::Tokenize)
                    .map_err(|e| format!("Token to string error: {:?}", e))?;
                debug!("Model output token: {}", output);
                full_response.push_str(&output);
    
                if options.as_ref().map_or(false, |opts| opts.stream) {
                    let payload = json!({
                        "chat_id": chat_id,
                        "output": json!({
                            "model": model,
                            "created_at": chrono::Utc::now().to_rfc3339(),
                            "message": {
                                "role": "assistant",
                                "content": output
                            },
                            "done": false
                        }).to_string() + "\n"
                    });
                    let _ = app.emit("model-stream-output", payload.to_string());
                }
    
                batch.clear();
                batch
                    .add(token, n_cur as i32, &[0], true)
                    .map_err(|e| format!("Batch addition error: {:?}", e))?;
                ctx.decode(&mut batch)
                    .map_err(|e| format!("Decoding failed: {:?}", e))?;
                n_cur += 1;
            }
    
            if !options.as_ref().map_or(false, |opts| opts.stream) {
                let output_value = match serde_json::from_str::<serde_json::Value>(&full_response) {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        let payload = json!({
                            "chat_id": chat_id,
                            "output": {
                                "model": model,
                                "created_at": chrono::Utc::now().to_rfc3339(),
                                "message": {
                                    "role": "assistant",
                                    "content": full_response
                                },
                                "done": true
                            }
                        });
                        let _ = app.emit("model-output", payload.to_string());
                        return Ok(());
                    }
                };
    
                if let (Some(name), Some(arguments)) = (
                    output_value.get("name").and_then(|v| v.as_str()),
                    output_value.get("parameters").cloned(),
                ) {
                    if allowed_function_names.contains(&name.to_string()) {
                        info!("Received function call: {}", name);
                        if let Some(func) = all_functions.get(name) {
                            let tool_result = futures::executor::block_on(func.call(arguments));
                            let tool_response = json!({
                                "chat_id": chat_id,
                                "output": {
                                    "role": "tool",
                                    "content": format!("Результат выполнения функции {name}: {}", tool_result.unwrap_or_default()),
                                    "tool_call_id": name
                                }
                            });
                            let _ = app.emit("model-output", tool_response.to_string());
                            return Ok(());
                        }
                    }
                }
    
                let payload = json!({
                    "chat_id": chat_id,
                    "output": {
                        "model": model,
                        "created_at": chrono::Utc::now().to_rfc3339(),
                        "message": {
                            "role": "assistant",
                            "content": output_value
                        },
                        "done": true
                    }
                });
                let _ = app.emit("model-output", payload.to_string());
            }
    
            Ok(())
        })
        .await
        .map_err(|e| format!("Failed to run model: {:?}", e))?;
    
        let mut running = self.running.lock().unwrap();
        *running = false;
    
        match &result {
            Ok(_) => info!("Model run completed successfully"),
            Err(e) => error!("Model run failed: {}", e),
        }
    
        result
    } */

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
