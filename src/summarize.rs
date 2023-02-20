use std::collections::HashMap;
use std::sync::Arc;

use crate::llms::llm_client::LlmClient;
use crate::util;
use crate::{prompt::format_prompt, settings::PromptSettings};
use anyhow::Result;
use tokio::task::JoinSet;
use tokio::try_join;
#[derive(Debug, Clone)]
pub(crate) struct SummarizationClient {
    client: Arc<dyn LlmClient>,

    prompt_file_diff: String,
    prompt_commit_summary: String,
    prompt_commit_title: String,
}

impl SummarizationClient {
    pub(crate) fn new(settings: PromptSettings, client: Box<dyn LlmClient>) -> Result<Self> {
        let prompt_file_diff = settings.file_diff.unwrap_or_default();
        let prompt_commit_summary = settings.commit_summary.unwrap_or_default();
        let prompt_commit_title = settings.commit_title.unwrap_or_default();

        Ok(Self {
            client: client.into(),
            prompt_file_diff,
            prompt_commit_summary,
            prompt_commit_title,
        })
    }

    pub(crate) async fn get_commit_message(&self, file_diffs: Vec<&str>) -> Result<String> {
        let mut set = JoinSet::new();

        for file_diff in file_diffs {
            let file_diff = file_diff.to_owned();
            let cloned_self = self.clone();
            set.spawn(async move { cloned_self.process_file_diff(&file_diff).await });
        }

        let mut summary_for_file: HashMap<String, String> = HashMap::with_capacity(set.len());
        while let Some(res) = set.join_next().await {
            if let Some((k, v)) = res.unwrap() {
                summary_for_file.insert(k, v);
            }
        }

        let summary_points = &summary_for_file
            .iter()
            .map(|(file_name, completion)| format!("[{file_name}]\n{completion}"))
            .collect::<Vec<String>>()
            .join("\n");

        let (title, completion) = try_join!(
            self.commit_title(summary_points),
            self.commit_summary(summary_points)
        )?;

        let mut message = String::with_capacity(1024);

        message.push_str(&format!("{title}\n\n{completion}\n\n"));
        for (file_name, completion) in &summary_for_file {
            if !completion.is_empty() {
                message.push_str(&format!("[{file_name}]\n{completion}\n"));
            }
        }

        // split message into lines and uniquefy lines
        let mut lines = message.lines().collect::<Vec<&str>>();
        lines.dedup();
        let message = lines.join("\n");

        Ok(message)
    }

    /// Splits the contents of a git diff by file.
    ///
    /// The file path is the first string in the returned tuple, and the
    /// file content is the second string in the returned tuple.
    ///
    /// The function assumes that the file_diff input is well-formed
    /// according to the Diff format described in the Git documentation:
    /// https://git-scm.com/docs/git-diff
    async fn process_file_diff(&self, file_diff: &str) -> Option<(String, String)> {
        if let Some(file_name) = util::get_file_name_from_diff(file_diff) {
            let completion = self.diff_summary(file_name, file_diff).await;
            Some((
                file_name.to_string(),
                completion.unwrap_or_else(|_| "".to_string()),
            ))
        } else {
            None
        }
    }

    async fn diff_summary(&self, file_name: &str, file_diff: &str) -> Result<String> {
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
