use std::collections::HashMap;

use crate::{
    openai::OpenAIClient,
    prompt::{
        format_prompt, PROMPT_TO_SUMMARIZE_DIFF, PROMPT_TO_SUMMARIZE_DIFF_SUMMARIES,
        PROMPT_TO_SUMMARIZE_DIFF_TITLE,
    },
};
use anyhow::Result;

pub(crate) async fn diff_summary(
    client: &OpenAIClient,
    file_name: &str,
    file_diff: &str,
) -> Result<String> {
    debug!("summarizing file: {}", file_name);

    let prompt = format_prompt(
        PROMPT_TO_SUMMARIZE_DIFF,
        HashMap::from([("FILE_DIFF", file_diff)]),
    )?;

    let completion = client.completions(&prompt).await;
    completion
}

pub(crate) async fn commit_summary(client: &OpenAIClient, summary_points: &str) -> Result<String> {
    let prompt = format_prompt(
        PROMPT_TO_SUMMARIZE_DIFF_SUMMARIES,
        HashMap::from([("SUMMARY_POINTS", summary_points)]),
    )?;

    let completion = client.completions(&prompt).await;
    completion
}

pub(crate) async fn commit_title(client: &OpenAIClient, summary_points: &str) -> Result<String> {
    let prompt = format_prompt(
        PROMPT_TO_SUMMARIZE_DIFF_TITLE,
        HashMap::from([("SUMMARY_POINTS", summary_points)]),
    )?;

    let completion = client.completions(&prompt).await;
    completion
}
