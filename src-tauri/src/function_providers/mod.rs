pub mod time_provider;
pub mod web_page_reader;

use tauri::AppHandle;

use crate::LlmFunction;
use std::sync::Arc;

/// Возвращает все доступные функции как Arc
pub fn all_functions(app: &AppHandle, embed_path: &str) -> Vec<Arc<dyn LlmFunction>> {
    vec![
        Arc::new(time_provider::UnixTimeFunction),
        Arc::new(web_page_reader::WebPageReaderFunction::new(
            app.clone(),
            embed_path.to_string(),
            /*segment_size=*/ 512,
            /*top_n=*/ 3,
        )),
        // Добавляй другие функции здесь
    ]
}
