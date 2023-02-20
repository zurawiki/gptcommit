use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait LlmClient {
    /// It takes a prompt as input, and returns the completion using an external Large Language Model.
    async fn completions(&self, prompt: &str) -> Result<String>;
}
