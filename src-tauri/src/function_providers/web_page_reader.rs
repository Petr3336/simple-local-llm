use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use uuid::Uuid;
use crate::function_provider::{FunctionDefinition, FunctionParam, LlmFunction};
use tauri::AppHandle;
use reqwest::Client;

pub struct WebPageReaderFunction {
    app: AppHandle,
    /// путь к модели (чтобы retrieve_context знал, какой модельный файл использовать)
    model_path: String,
    segment_size: u32,
    top_n: usize,
}

impl WebPageReaderFunction {
    pub fn new(app: AppHandle, model_path: String, segment_size: u32, top_n: usize) -> Self {
        Self { app, model_path, segment_size, top_n }
    }
}

#[async_trait]
impl LlmFunction for WebPageReaderFunction {
    fn definition(&self) -> FunctionDefinition {
        // Описываем сигнатуру функции для LLM
        let mut params = HashMap::new();
        params.insert("url".into(), FunctionParam {
            name: "url".into(),
            description: "Адрес веб-страницы для анализа. Обязателен".into(),
            param_type: "string".into(),
        });
        params.insert("query".into(), FunctionParam {
            name: "query".into(),
            description: "Текст, по которому искать релевантные фрагменты. Обязателен".into(),
            param_type: "string".into(),
        });
        FunctionDefinition {
            name: "analyze_web_page".into(),
            description: Some("Web Page reader function".into()),
            parameters: params,
        }
    }

    async fn call(&self, args: Value) -> Result<Value, String> {
        // 1. Вытащим параметры
        let url = args.get("url")
            .and_then(Value::as_str)
            .ok_or("Missing parameter 'url'")?;
        let query = args.get("query")
            .and_then(Value::as_str)
            .ok_or("Missing parameter 'query'")?;

        let client = Client::new();
        // 2. Составляем URL Jina.ai Reader
        let reader_api = format!("https://r.jina.ai/{}", url);
        // 3. GET-запрос с нужным заголовком
        let content = client
            .get(&reader_api)
            .header("X-Retain-Images", "none")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        // 4. Сохраняем его во временный файл
        let fname = format!("page_{}.txt", Uuid::new_v4());
        let tmp = std::env::temp_dir().join(&fname);
        fs::write(&tmp, &content).map_err(|e| e.to_string())?;

        // 5. Вызываем вашу функцию retrieve_context
        let relevant = crate::embeddings::retrieve_context(
            self.app.clone(),
            self.model_path.clone(),
            query.to_string(),
            vec![ tmp.to_string_lossy().to_string() ],
            self.segment_size,
            self.top_n,
        )
        .await
        .map_err(|e| e.to_string())?;

        // 6. Возвращаем результат
        Ok(json!({ "relevant_text": relevant }))
    }
}
