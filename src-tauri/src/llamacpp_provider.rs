use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;

use llama_cpp_2::{
    Model, ModelParameters, Session, SessionParams, InferenceParams, InferenceResult,
};

use crate::model_provider::{ModelProvider, LLMOptions};

pub struct LlamaCppProvider {
    model_path: PathBuf,
    model: Mutex<Option<Model>>,
}

impl LlamaCppProvider {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            model: Mutex::new(None),
        }
    }

    fn load_model(&self) -> Result<Model, String> {
        Model::load(&self.model_path, &ModelParameters::default())
            .map_err(|e| format!("Ошибка загрузки модели: {:?}", e))
    }
}

#[async_trait]
impl ModelProvider for LlamaCppProvider {
    fn name(&self) -> &'static str {
        "llama-cpp"
    }

    async fn get_installed_models(&self) -> Result<Vec<String>, String> {
        Ok(vec![self.model_path.to_string_lossy().to_string()])
    }

    async fn run_model(
        &self,
        app: AppHandle,
        _model: String,
        prompt: String,
        _options: Option<LLMOptions>,
    ) -> Result<(), String> {
        let model = {
            let mut guard = self.model.lock().unwrap();
            if let Some(model) = &*guard {
                model.clone()
            } else {
                let loaded = self.load_model()?;
                *guard = Some(loaded.clone());
                loaded
            }
        };

        let session = Session::new(&model, &SessionParams::default())
            .map_err(|e| format!("Ошибка создания сессии: {:?}", e))?;

        let mut stream = session.infer(
            &prompt,
            &InferenceParams {
                stream: true,
                ..Default::default()
            },
        ).map_err(|e| format!("Ошибка инференса: {:?}", e))?;

        tauri::async_runtime::spawn(async move {
            while let Some(res) = stream.next().await {
                match res {
                    Ok(InferenceResult::Token(token)) => {
                        let _ = app.emit("ollama-output", token);
                    }
                    Ok(_) => {}
                    Err(e) => {
                        let _ = app.emit("ollama-output", format!("[Ошибка: {:?}]", e));
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}
