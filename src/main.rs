mod com;

use com::llm::server::core::model::{LlmQuery, LlmAnswer};
use com::llm::server::core::rest_server::start_rest_server;

fn main() {
    println!("Hello, world!");
    // other functions within your bootstrap module
    start_rest_server();

    let query = LlmQuery {
        prompt: "Write a haiku about a tree".to_string(),
    };
}
