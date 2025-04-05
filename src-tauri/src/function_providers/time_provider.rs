use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use serde_json::{json, Value};

use crate::function_provider::{LlmFunction, FunctionDefinition};
use async_trait::async_trait;

pub struct UnixTimeFunction;

#[async_trait]
impl LlmFunction for UnixTimeFunction {
    fn definition(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: "get_unix_time".to_string(),
            description: Some("Returns current UNIX timestamp.".to_string()),
            parameters: HashMap::new(),
        }
    }

    async fn call(&self, _args: Value) -> Result<Value, String> {
        let now = SystemTime::now();
        match now.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                Ok(json!({ "unix_time": secs }))
            }
            Err(e) => Err(format!("Ошибка получения времени: {}", e)),
        }
    }
}
