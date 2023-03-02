use anyhow::{anyhow, bail, Result};

use async_trait::async_trait;

use tiktoken_rs::tiktoken::{p50k_base, CoreBPE};

use crate::settings::OpenAISettings;
use async_openai::{types::CreateCompletionRequestArgs, Client};

use super::llm_client::LlmClient;

#[derive(Clone, Debug)]
pub(crate) struct OpenAIClient {
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

        let client = Client::new().with_api_key(&api_key);
        Ok(Self { model, client })
    }

    pub(crate) fn get_prompt_token_limit_for_model(&self) -> usize {
        match self.model.as_str() {
            "text-davinci-003" => 4000,
            "text-davinci-002" => 4000,
            "text-curie-001" => 2048,
            "text-babbage-001" => 2048,
            "text-ada-001" => 2048,
            "code-davinci-002" => 4000,
            "code-cushman-001" => 2048,
            _ => 4096,
        }
    }
}

#[async_trait]
impl LlmClient for OpenAIClient {
    /// Sends a request to OpenAI's API to get a text completion.
    /// It takes a prompt as input, and returns the completion.
    async fn completions(&self, prompt: &str) -> Result<String> {
        let prompt_token_limit = self.get_prompt_token_limit_for_model();
        lazy_static! {
            static ref BPE_TOKENIZER: CoreBPE = p50k_base().unwrap();
        }
        let output_length = 100;

        let tokens = BPE_TOKENIZER.encode_with_special_tokens(prompt);
        let prompt_token_count = tokens.len();
        if prompt_token_count + output_length > prompt_token_limit {
            let error_msg =
                format!("skipping... token count: {prompt_token_count} < {prompt_token_limit}");
            warn!("{}", error_msg);
            bail!(error_msg)
        }

        // Create request using builder pattern
        let request = CreateCompletionRequestArgs::default()
            .model(&self.model)
            .prompt(prompt)
            .max_tokens(output_length as u16)
            .temperature(0.5)
            .top_p(1.)
            .frequency_penalty(0.)
            .presence_penalty(0.)
            .build()?;

        debug!("Sending request to OpenAI:\n{:?}", request);

        let response = self
            .client
            .completions() // Get the API "group" (completions, images, etc.) from the client
            .create(request) // Make the API call in that "group"
            .await?;

        let completion = response
            .choices
            .first()
            .ok_or(anyhow!("No completion results"))
            .map(|c| c.text.clone());

        return completion;
    }
}
