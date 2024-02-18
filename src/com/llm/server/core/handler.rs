use std::convert::Infallible;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, Write, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use hyper::body::Body;
use hyper::http::{Request, Response};
use serde::{Deserialize, Serialize};
use crate::com::llm::server::core::model::{LlmAnswer, LlmQuery};

/// Configuration struct to hold model information.
#[derive(Debug, Deserialize)]
struct ModelConfig {
    model_name: String,
}

/// Load model configuration from a JSON file.
fn load_model_config() -> Result<ModelConfig, Box<dyn Error>> {
    let config_path = Path::new("./config.json");
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: ModelConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

/// Perform language model inference.
fn infer(prompt: &str, model_name: &str) -> Result<String, Box<dyn Error>> {
    let tokenizer_source = llm::TokenizerSource::Embedded;
    let model_architecture = llm::ModelArchitecture::Llama;
    let model_path = PathBuf::from(model_name);
    let prompt = prompt.to_string();
    let now = std::time::Instant::now();
    let model = llm::load_dynamic(
        Some(model_architecture),
        &model_path,
        tokenizer_source,
        Default::default(),
        llm::load_progress_callback_stdout,
    )?;

    println!(
        "Model fully loaded! Elapsed: {}ms",
        now.elapsed().as_millis()
    );

    let mut session = model.start_session(Default::default());
    let mut generated_tokens = String::new(); // Accumulate generated tokens here

    let res = session.infer::<Infallible>(
        model.as_ref(),
        &mut rand::thread_rng(),
        &llm::InferenceRequest {
            prompt: (&prompt).into(),
            parameters: &llm::InferenceParameters::default(),
            play_back_previous_tokens: false,
            maximum_token_count: Some(140),
        },
        // OutputRequest
        &mut Default::default(),
        |r| match r {
            llm::InferenceResponse::PromptToken(t) | llm::InferenceResponse::InferredToken(t) => {
                print!("{t}");
                std::io::stdout().flush().unwrap();
                // Accumulate generated tokens
                generated_tokens.push_str(&t);
                Ok(llm::InferenceFeedback::Continue)
            }
            _ => Ok(llm::InferenceFeedback::Continue),
        },
    );

    // Return the accumulated generated tokens
    match res {
        Ok(_) => Ok(generated_tokens),
        Err(err) => Err(Box::new(err)),
    }
}

/// Handles the LLM query request.
pub(crate) async fn llm_query(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
    let chat_request: Result<LlmQuery, _> = serde_json::from_slice(&body_bytes);
    match chat_request {
        Ok(chat_request) => {
            match load_model_config() {
                Ok(config) => {
                    match infer(&chat_request.prompt, &config.model_name) {
                        Ok(inference_result) => {
                            let response_message = format!("Inference result: {}", inference_result);
                            let chat_response = LlmAnswer { response: response_message };
                            let response = Response::new(Body::from(serde_json::to_string(&chat_response).unwrap()));
                            Ok(response)
                        }
                        Err(err) => {
                            eprintln!("Error in inference: {:?}", err);
                            Ok(Response::builder().status(500).body(Body::empty()).unwrap())
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Error loading model configuration.");
                    Ok(Response::builder().status(500).body(Body::empty()).unwrap())
                }
            }
        }
        Err(_) => {
            Ok(Response::builder().status(400).body(Body::empty()).unwrap())
        }
    }
}