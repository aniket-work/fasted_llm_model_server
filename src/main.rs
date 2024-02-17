mod com;

use com::llm::server::core::model::{LlmQuery, LlmAnswer};
use com::llm::server::core::rest_server::start_rest_server;

#[tokio::main]
async fn main() {
    println!("LLM Server Queue initiated...");
    // other functions within your bootstrap module
    start_rest_server().await;

    let query = LlmQuery {
        prompt: "Howz the day Today? ".to_string(),
    };
}
