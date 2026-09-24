use anyhow::{anyhow, bail, Ok, Result};
use std::fmt;
use std::fmt::Debug;
use std::time::Duration;

use async_trait::async_trait;

use reqwest::{tls, Proxy};
use tiktoken_rs::{get_chat_completion_max_tokens, get_completion_max_tokens};

const DEFAULT_MAX_TOKENS: usize = 4096;

use crate::{settings::OpenAISettings, util::HTTP_USER_AGENT};
use async_openai::{
    config::{OpenAIConfig, OPENAI_API_BASE},
    middleware::{retry::OpenAIRetryLayer, ReqwestService},
    types::{
        chat::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
        completions::CreateCompletionRequestArgs,
    },
    Client,
};

use super::llm_client::LlmClient;
const COMPLETION_TOKEN_LIMIT: usize = 100;

pub(crate) struct OpenAIClient {
    model: String,
    client: Client<OpenAIConfig>,
}

impl Debug for OpenAIClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAIClient")
            .field("model", &self.model)
            .finish()
    }
}

impl OpenAIClient {
    pub(crate) fn new(settings: OpenAISettings) -> Result<Self, anyhow::Error> {
        let api_base = settings
            .api_base
            .unwrap_or_else(|| OPENAI_API_BASE.to_string());
        let api_key = settings.api_key.unwrap_or_default();

        let openai_config = OpenAIConfig::new()
            .with_api_base(&api_base)
            .with_api_key(&api_key);

        let mut openai_client = Client::<OpenAIConfig>::with_config(openai_config);

        if api_base == OPENAI_API_BASE && api_key.is_empty() {
            bail!("No OpenAI API key found. Please provide a valid API key.");
        }
        // TODO make configurable
        let mut http_client = reqwest::Client::builder()
            .gzip(true)
            .brotli(true)
            .timeout(Duration::from_secs(60))
            .user_agent(HTTP_USER_AGENT);

        if api_base == OPENAI_API_BASE {
            // Optimized HTTP client
            http_client = http_client
                .http2_prior_knowledge()
                .https_only(true)
                .http2_adaptive_window(true)
                .tcp_keepalive(Duration::from_secs(60))
                .http2_keep_alive_interval(Duration::from_secs(60))
                .http2_keep_alive_while_idle(true)
                .min_tls_version(tls::Version::TLS_1_2);
        }
        let model = settings.model.unwrap_or_default();
        if api_base == OPENAI_API_BASE && model.is_empty() {
            bail!("No OpenAI model configured. Please choose a valid model to use.");
        }

        if let Some(proxy) = settings.proxy {
            if !proxy.is_empty() {
                http_client = http_client.proxy(Proxy::all(proxy)?);
            }
        }
        let service = tower::ServiceBuilder::new()
            .layer(OpenAIRetryLayer::new(usize::from(
                settings.retries.unwrap_or_default(),
            )))
            .service(ReqwestService::new(http_client.build()?));
        openai_client = openai_client.with_http_service(service);
        Ok(Self {
            model,
            client: openai_client,
        })
    }

    pub(crate) fn should_use_chat_completion(model: &str) -> bool {
        let model = model.to_lowercase();
        // Only use the legacy completions API for known old models
        let legacy_models = [
            "text-davinci",
            "text-curie",
            "text-babbage",
            "text-ada",
            "code-",
        ];
        !legacy_models.iter().any(|prefix| model.starts_with(prefix))
    }

    pub(crate) async fn get_completions(&self, prompt: &str) -> Result<String> {
        let prompt_token_limit =
            get_completion_max_tokens(&self.model, prompt).unwrap_or_else(|_| {
                warn!(
                    "Unknown model '{}' for token counting, using default limit",
                    self.model
                );
                DEFAULT_MAX_TOKENS
            });

        if prompt_token_limit < COMPLETION_TOKEN_LIMIT {
            let error_msg =
"Skipping... The diff is too large for the current model. Consider using a model with a larger context window.".to_string();
            warn!("{}", error_msg);
            bail!(error_msg)
        }
        // Create request using builder pattern
        let request = CreateCompletionRequestArgs::default()
            .model(&self.model)
            .prompt(prompt)
            .max_tokens(prompt_token_limit as u32)
            .temperature(0.5_f32)
            .top_p(1.0_f32)
            .frequency_penalty(0.0_f32)
            .presence_penalty(0.0_f32)
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
            .ok_or(anyhow!("No completion results returned from OpenAI."))
            .map(|c| c.text.clone());

        completion
    }

    pub(crate) async fn get_chat_completions(&self, prompt: &str) -> Result<String> {
        let message = ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?;
        let token_messages = [tiktoken_rs::ChatCompletionRequestMessage {
            role: "user".to_string(),
            content: Some(prompt.to_string()),
            ..Default::default()
        }];
        let prompt_token_limit = get_chat_completion_max_tokens(&self.model, &token_messages)
            .unwrap_or_else(|_| {
                warn!(
                    "Unknown model '{}' for token counting, using default limit",
                    self.model
                );
                DEFAULT_MAX_TOKENS
            });

        if prompt_token_limit < COMPLETION_TOKEN_LIMIT {
            let error_msg =
                "skipping... diff is too large for the model. Consider using a model with a larger context window.".to_string();
            warn!("{}", error_msg);
            bail!(error_msg)
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(message)
            .build()?;

        let response = self.client.chat().create(request).await?;

        if let Some(choice) = response.choices.into_iter().next() {
            debug!(
                "{}: Role: {}  Content: {}",
                choice.index,
                choice.message.role,
                choice.message.content.clone().unwrap_or_default()
            );

            return choice
                .message
                .content
                .ok_or(anyhow!("No completion results returned from OpenAI."));
        }

        bail!("No completion results returned from OpenAI.")
    }
}

#[async_trait]
impl LlmClient for OpenAIClient {
    /// Sends a request to OpenAI's API to get a text completion.
    /// It takes a prompt as input, and returns the completion.
    async fn completions(&self, prompt: &str) -> Result<String> {
        let completion = if OpenAIClient::should_use_chat_completion(&self.model) {
            self.get_chat_completions(prompt).await?
        } else {
            self.get_completions(prompt).await?
        };
        Ok(completion.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn serve(
        responses: Vec<(u16, Value)>,
    ) -> (String, tokio::task::JoinHandle<Vec<(String, Value)>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let mut requests = Vec::new();
            for (status, body) in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let (headers, body_start, length) = loop {
                    let mut buffer = [0; 4096];
                    let read = socket.read(&mut buffer).await.unwrap();
                    assert_ne!(read, 0);
                    bytes.extend_from_slice(&buffer[..read]);
                    if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let headers = String::from_utf8(bytes[..end].to_vec()).unwrap();
                        let length = headers
                            .lines()
                            .find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().unwrap())
                            })
                            .unwrap();
                        break (headers, end + 4, length);
                    }
                };
                while bytes.len() < body_start + length {
                    let mut buffer = [0; 4096];
                    let read = socket.read(&mut buffer).await.unwrap();
                    assert_ne!(read, 0);
                    bytes.extend_from_slice(&buffer[..read]);
                }
                requests.push((
                    headers.lines().next().unwrap().to_string(),
                    serde_json::from_slice(&bytes[body_start..body_start + length]).unwrap(),
                ));
                let body = body.to_string();
                let response = format!(
                    "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nRetry-After: 0\r\n\r\n{body}",
                    body.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
            requests
        });
        (address, task)
    }

    fn response(chat: bool) -> Value {
        let choice = if chat {
            json!({"index": 0, "message": {"role": "assistant", "content": " summary "}, "finish_reason": "stop"})
        } else {
            json!({"index": 0, "text": " summary ", "logprobs": null, "finish_reason": "stop"})
        };
        json!({"id": "test", "object": "completion", "created": 0, "model": "test", "choices": [choice]})
    }

    #[tokio::test]
    async fn completion_routes_preserve_request_and_response() {
        for (model, chat, path) in [
            ("custom-chat-model", true, "/chat/completions"),
            ("text-davinci-003", false, "/completions"),
        ] {
            let (api_base, server) = serve(vec![(200, response(chat))]).await;
            let client = OpenAIClient::new(OpenAISettings {
                api_base: Some(api_base),
                model: Some(model.to_string()),
                ..Default::default()
            })
            .unwrap();
            let result =
                tokio::time::timeout(Duration::from_secs(5), client.completions("test prompt"))
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!(result, "summary");
            let requests = server.await.unwrap();
            assert_eq!(requests[0].0, format!("POST {path} HTTP/1.1"));
            assert_eq!(requests[0].1["model"], model);
            if chat {
                assert_eq!(
                    requests[0].1["messages"],
                    json!([{"role": "user", "content": "test prompt"}])
                );
            } else {
                assert_eq!(requests[0].1["prompt"], "test prompt");
                assert_eq!(requests[0].1["temperature"], 0.5);
            }
        }
    }

    #[tokio::test]
    async fn retries_respect_configured_count() {
        for retries in [0, 1] {
            let error = json!({"error": {"message": "slow down", "type": "rate_limit_error"}});
            let mut responses = vec![(429, error)];
            if retries > 0 {
                responses.push((200, response(true)));
            }
            let (api_base, server) = serve(responses).await;
            let client = OpenAIClient::new(OpenAISettings {
                api_base: Some(api_base),
                model: Some("custom-chat-model".to_string()),
                retries: Some(retries),
                ..Default::default()
            })
            .unwrap();
            let result = tokio::time::timeout(Duration::from_secs(5), client.completions("test"))
                .await
                .unwrap();
            assert_eq!(result.is_ok(), retries > 0);
            assert_eq!(server.await.unwrap().len(), usize::from(retries) + 1);
        }
    }
}
