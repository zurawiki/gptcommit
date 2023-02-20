use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LlmClient: Debug + Send + Sync {
    /// It takes a prompt as input, and returns the completion using an external Large Language Model.
    async fn completions(&self, prompt: &str) -> Result<String>;
}
