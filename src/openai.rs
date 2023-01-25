use std::time::Duration;

use anyhow::{anyhow, bail, Result};

use reqwest::{Client, ClientBuilder};
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

        let timeout = Duration::new(15, 0);
        let client = ClientBuilder::new().timeout(timeout).build()?;
        Ok(Self {
            api_key,
            model,
            client,
        })
    }

    /// Sends a request to OpenAI's API to get a text completion.
    /// It takes a prompt as input, and returns the completion.
    pub(crate) async fn completions(&self, prompt: &str) -> Result<String> {
        let prompt_token_limit = self.get_prompt_token_limit_for_model();
        // TODO use real tokenizer. Right now we assume, 1 word = 2 tokens
        let prompt_token_count = prompt.split_whitespace().filter(|s| !s.is_empty()).count() * 2;

        if prompt_token_count > prompt_token_limit {
            let error_msg = format!(
                "skipping... token count: {} < {}",
                prompt_token_count, prompt_token_limit
            );
            warn!("{}", error_msg);
            bail!(error_msg)
        }

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

    pub(crate) fn get_prompt_token_limit_for_model(&self) -> usize {
        match self.model.as_str() {
            "text-davinci-003" => 4000,
            _ => 2048,
        }
    }
}
