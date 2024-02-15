use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
pub struct LlmQuery {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct LlmAnswer {
    pub response: String,
}
