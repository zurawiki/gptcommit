use std::collections::HashMap;

use crate::{llms::openai::OpenAIClient, prompt::format_prompt, settings::PromptSettings};
use anyhow::Result;

#[derive(Clone, Debug)]
pub(crate) struct SummarizationClient {
    client: OpenAIClient,

    prompt_file_diff: String,
    prompt_commit_summary: String,
    prompt_commit_title: String,
}

impl SummarizationClient {
    pub(crate) fn new(
        settings: PromptSettings,
        client: OpenAIClient,
    ) -> Result<Self, anyhow::Error> {
        let prompt_file_diff = settings.file_diff.unwrap_or_default();
        let prompt_commit_summary = settings.commit_summary.unwrap_or_default();
        let prompt_commit_title = settings.commit_title.unwrap_or_default();

        Ok(Self {
            client,
            prompt_file_diff,
            prompt_commit_summary,
            prompt_commit_title,
        })
    }

    pub(crate) async fn diff_summary(&self, file_name: &str, file_diff: &str) -> Result<String> {
        debug!("summarizing file: {}", file_name);

        let prompt = format_prompt(
            &self.prompt_file_diff,
            HashMap::from([("file_diff", file_diff)]),
        )?;

        let completion = self.client.completions(&prompt).await;
        completion
    }

    pub(crate) async fn commit_summary(&self, summary_points: &str) -> Result<String> {
        let prompt = format_prompt(
            &self.prompt_commit_summary,
            HashMap::from([("summary_points", summary_points)]),
        )?;

        let completion = self.client.completions(&prompt).await;
        completion
    }

    pub(crate) async fn commit_title(&self, summary_points: &str) -> Result<String> {
        let prompt = format_prompt(
            &self.prompt_commit_title,
            HashMap::from([("summary_points", summary_points)]),
        )?;

        let completion = self.client.completions(&prompt).await;
        completion
    }
}
