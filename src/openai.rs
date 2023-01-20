use anyhow::{anyhow, Result};

use reqwest::Client;
use serde_json::{json, Value};

/// Sends a request to OpenAI's API to get a text completion.
/// It takes a prompt as input, and returns the completion.
pub(crate) async fn completions(prompt: &str) -> Result<String> {
    let api_key = get_openai_api_key()?;
    let client = Client::new();

    let json_data = json!({
        "model": "text-davinci-003",
        "prompt": prompt,
        "temperature": 0.5,
        "max_tokens": 100,
        "top_p": 1,
        "frequency_penalty": 0,
        "presence_penalty": 0
    });
    let request = client
        .post("https://api.openai.com/v1/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json_data);

    debug!("Sending request to OpenAI:\n{:?}", request);

    let response = request.send().await?;
    let response_body = response.text().await?;
    let json_response: Value = serde_json::from_str(&response_body).map_err(|e| {
        anyhow!(
            "Could not decode JSON response: {}\nResponse body: {:?}",
            e,
            response_body
        )
    })?;
    Ok(json_response["choices"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow!("Unexpected JSON response:\n{}", json_response))?
        .to_string())
}

pub(crate) fn get_openai_api_key() -> Result<String> {
    // TODO Ask for API key and save it (or confirm from env, flag)
    std::env::var("OPENAI_API_KEY").map_err(|e| anyhow!("Could not find OPENAI_API_KEY: {}", e))
}
