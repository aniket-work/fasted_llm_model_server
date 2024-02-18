use std::fs;
use std::fs::File;
use std::io::copy;

use std::path::Path;


pub(crate) fn download_model() -> Result<(), Box<dyn std::error::Error>> {
    let token = "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";
    let model_id = "rustformers/redpajama-3b-ggml";
    let model_url = format!("https://huggingface.co/{}/resolve/main/config.json", model_id);
    let dest_path = format!("models/{}/config.json", model_id.replace("/", "_"));

    // Create the parent directories if they don't exist
    let parent_dir = Path::new(&dest_path).parent().unwrap();
    fs::create_dir_all(parent_dir)?;

    // Now proceed with creating the file
    let mut file = File::create(&dest_path)?;

    // Use the blocking reqwest client
    let client = reqwest::blocking::Client::new();
    let response = client.get(&model_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    if response.status().is_success() {
        let mut file = File::create(&dest_path)?;
        copy(&mut response.bytes()?.as_ref(), &mut file)?;
        println!("Model downloaded successfully!");
    } else {
        println!("Failed to download model: {:?}", response.status());
    }

    Ok(())
}

