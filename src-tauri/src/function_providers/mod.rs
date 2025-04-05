pub mod time_provider;

use crate::LlmFunction;
use std::sync::Arc;

/// Возвращает все доступные функции как Arc
pub fn all_functions() -> Vec<Arc<dyn LlmFunction>> {
    vec![
        Arc::new(time_provider::UnixTimeFunction),
        // Добавляй другие функции здесь
    ]
}