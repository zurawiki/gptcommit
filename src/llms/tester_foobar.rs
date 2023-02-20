use anyhow::Result;

use async_trait::async_trait;

#[cfg(test)]
use async_std::task;

use super::llm_client::LlmClient;

#[derive(Clone, Debug)]
/// Tester LLM client
pub(crate) struct FooBarClient {}

impl FooBarClient {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl LlmClient for FooBarClient {
    /// Dummy Completion that responds with "foo bar" for prompt
    async fn completions(&self, _prompt: &str) -> Result<String> {
        Ok("foo bar".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        task::block_on(async {
            let client = FooBarClient::new().unwrap();

            let result = client.completions("Hi there! ").await.unwrap();
            assert_eq!(result, "foo bar");
        });
    }
}
