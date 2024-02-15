use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};


use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, io::Write, path::PathBuf};
use hyper::http::uri::Builder;


#[derive(Debug, Deserialize)]
struct ChatRequest {
    prompt: String,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    response: String,
}

async fn chat_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
    let chat_request: Result<ChatRequest, _> = serde_json::from_slice(&body_bytes);

    match chat_request {
        Ok(chat_request) => {
            // Call the `infer` function with the received prompt
            match infer(chat_request.prompt) {
                Ok(inference_result) => {
                    // Prepare the response message
                    let response_message = format!("Inference result: {}", inference_result);
                    let chat_response = ChatResponse {
                        response: response_message,
                    };
                    // Serialize the response and send it back
                    let response = Response::new(Body::from(serde_json::to_string(&chat_response).unwrap()));
                    Ok(response)
                }
                Err(err) => {
                    eprintln!("Error in inference: {:?}", err);
                    // Return a 500 Internal Server Error response
                    Ok(Response::builder()
                        .status(500)
                        .body(Body::empty())
                        .unwrap())
                }
            }
        }
        Err(_) => {
            // Return a 400 Bad Request response for JSON deserialization failure
            Ok(Response::builder()
                .status(400)
                .body(Body::empty())
                .unwrap())
        }
    }
}

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.uri().path(), req.method()) {
        ("/api/chat", &hyper::Method::POST) => chat_handler(req).await,
        _ => not_found(),
    }
}

fn not_found() -> Result<Response<Body>, Infallible> {
    // Return a 404 Not Found response
    Ok(Response::builder()
        .status(404)
        .body(Body::empty())
        .unwrap())
}

pub async fn start_rest_server() {
    println!("Server listening on port 8083...");
    let addr = ([0, 0, 0, 0], 8083).into();
    let make_svc = hyper::service::make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(hyper::service::service_fn(router)) }
    });
    let server = Builder::new()
        .bind(&addr, make_svc)
        .expect("Failed to create server builder")
        .run();
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
