use anyhow::{anyhow, bail, Result};

use reqwest::Client;
use serde_json::{json, Value};

use crate::settings::OpenAISettings;

#[derive(Clone, Debug)]
pub(crate) struct OpenAIClient {
    api_key: String,
    model: String,
    client: Client,
}

impl OpenAIClient {
    pub(crate) fn new(settings: OpenAISettings) -> Result<Self, anyhow::Error> {
        let api_key = settings.api_key.unwrap_or_default();
        if api_key.is_empty() {
            bail!("No OpenAI API key found.")
        }
        let model = settings.model.unwrap_or_default();
        if model.is_empty() {
            bail!("No OpenAI model configured.")
        }
        Ok(Self {
            api_key,
            model,
            client: Client::new(),
        })
    }

    /// Sends a request to OpenAI's API to get a text completion.
    /// It takes a prompt as input, and returns the completion.
    pub(crate) async fn completions(&self, prompt: &str) -> Result<String> {
        let json_data = json!({
            "model": self.model,
            "prompt": prompt,
            "temperature": 0.5,
            "max_tokens": 100,
            "top_p": 1,
            "frequency_penalty": 0,
            "presence_penalty": 0
        });
        let request = self
            .client
            .post("https://api.openai.com/v1/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
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
}
