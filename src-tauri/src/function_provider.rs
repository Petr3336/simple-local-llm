use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub param_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: HashMap<String, FunctionParam>,
}

#[async_trait]
pub trait LlmFunction: Send + Sync {
    /// Возвращает метаописание функции
    fn definition(&self) -> FunctionDefinition;

    /// Вызывает функцию с JSON-параметрами
    async fn call(&self, args: Value) -> Result<Value, String>;
}
