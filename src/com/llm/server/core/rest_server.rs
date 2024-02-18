use com::llm::server::core::handler::llm_query;
use crate::com;

use cargo_metadata::{MetadataCommand};
use serde_json::{json, to_string};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::net::SocketAddr;
use std::convert::Infallible;
use anyhow::{Context, Error};
use std::fs;
use crate::com::llm::server::core::download_model::download_model;


// Function to read version details from Cargo.toml
fn read_cargo_toml() -> Result<Option<String>, Error> {
    // Read the contents of Cargo.toml
    let cargo_toml_content = fs::read_to_string("Cargo.toml")
        .with_context(|| "Failed to read Cargo.toml")?;

    // Parse the Cargo.toml content as TOML
    let cargo_toml: toml::Value = cargo_toml_content.parse()
        .with_context(|| "Failed to parse Cargo.toml")?;

    // Access the [package] table
    let package_table = cargo_toml.get("package")
        .ok_or_else(|| Error::msg("Missing [package] table in Cargo.toml"))?
        .as_table()
        .ok_or_else(|| Error::msg("Invalid [package] table in Cargo.toml"))?;

    // Read the individual fields under [package]
    let name = package_table.get("name")
        .and_then(|name| name.as_str())
        .map(|name| name.to_string());

    let version = package_table.get("version")
        .and_then(|version| version.as_str())
        .map(|version| version.to_string());

    let edition = package_table.get("edition")
        .and_then(|edition| edition.as_str())
        .map(|edition| edition.to_string());

    // Return the extracted fields
    Ok(Some(format!("name: {:?}, version: {:?}, edition: {:?}", name, version, edition)))
}


// Router function to handle different API endpoints
async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.uri().path(), req.method()) {
        ("/api/chat", &hyper::Method::POST) => {
            llm_query(req).await
        }
        ("/api/health", &hyper::Method::GET) => {
            // Return a 200 OK response with health information in JSON format
            let health_response = json!({
                "status": "OK",
                "details": "Server is running smoothly."
            });
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(to_string(&health_response).unwrap()))
                .unwrap())
        }
        ("/api/app/version", &hyper::Method::GET) => {
            // Read version details from Cargo.toml
            match read_cargo_toml() {
                Ok(Some(version)) => {
                    // Return a 200 OK response with version information in JSON format
                    let version_response = json!({
                        "version": version
                    });
                    Ok(Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .body(Body::from(to_string(&version_response).unwrap()))
                        .unwrap())
                }
                Ok(None) => {
                    // Return a 404 Not Found response if version not found
                    Ok(Response::builder()
                        .status(404)
                        .body(Body::from("Version not found"))
                        .unwrap())
                }
                Err(err) => {
                    // Return a 500 Internal Server Error if there is an error reading Cargo.toml
                    eprintln!("Error reading Cargo.toml: {}", err);
                    Ok(Response::builder()
                        .status(500)
                        .body(Body::from("Error reading Cargo.toml"))
                        .unwrap())
                }
            }
        }
        _ => {
            // Return a 404 Not Found response for undefined routes
            Ok(Response::builder()
                .status(404)
                .body(Body::from("Route not found"))
                .unwrap())
        }
    }
}


/// Handles cases where requested route is not found.
fn not_found() -> Result<Response<Body>, Infallible> {
    // Return a 404 Not Found response
    Ok(Response::builder()
        .status(404)
        .body(Body::empty())
        .unwrap())
}

/// Starts the REST server and listens for incoming requests.
pub(crate) async fn start_rest_server() {

    // Call the download_model function
    match download_model() {
        Ok(_) => println!("Model initialized and ready to server !"),
        Err(err) => eprintln!("Error downloading model: {:?}", err),
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], 8088));
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(router)) }
    });
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

}
