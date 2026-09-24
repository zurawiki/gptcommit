use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::Arc;

use crate::llms::llm_client::LlmClient;
use crate::settings::Settings;
use crate::util;
use crate::{prompt::format_prompt, settings::Language};
use anyhow::{bail, Context as _, Result};
use ignore::gitignore::GitignoreBuilder;

use tokio::task::JoinSet;
use tokio::try_join;

use tera::{Context, Tera};

pub(crate) fn filter_diffs<'a>(patterns: &[String], diffs: Vec<&'a str>) -> Result<Vec<&'a str>> {
    let mut builder = GitignoreBuilder::new("");
    for pattern in patterns {
        builder
            .add_line(None, pattern)
            .with_context(|| format!("Invalid file_ignore pattern: {pattern}"))?;
    }
    let matcher = builder.build()?;
    Ok(diffs
        .into_iter()
        .filter(|diff| {
            let Some(name) = util::get_file_name_from_diff(diff) else {
                return false;
            };
            if let ignore::Match::Ignore(rule) = matcher.matched_path_or_any_parents(name, false) {
                info!("skipping {name}: file_ignore pattern {:?}", rule.original());
                false
            } else {
                true
            }
        })
        .collect())
}

#[derive(Debug, Clone)]
pub(crate) struct SummarizationClient {
    client: Arc<dyn LlmClient>,

    prompt_file_diff: String,
    prompt_conventional_commit_prefix: String,
    prompt_commit_summary: String,
    prompt_commit_title: String,
    prompt_translation: String,
    output_conventional_commit: bool,
    output_conventional_commit_prefix_format: String,
    output_lang: Language,
    output_show_per_file_summary: bool,
}

impl SummarizationClient {
    pub(crate) fn new(settings: Settings, client: Box<dyn LlmClient>) -> Result<Self> {
        let prompt_settings = settings.prompt.unwrap_or_default();

        let prompt_file_diff = prompt_settings.file_diff.unwrap_or_default();
        let prompt_conventional_commit_prefix = prompt_settings
            .conventional_commit_prefix
            .unwrap_or_default();
        let prompt_commit_summary = prompt_settings.commit_summary.unwrap_or_default();
        let prompt_commit_title = prompt_settings.commit_title.unwrap_or_default();
        let prompt_translation = prompt_settings.translation.unwrap_or_default();

        let output_settings = settings.output.unwrap_or_default();
        let output_conventional_commit = output_settings.conventional_commit.unwrap_or(true);
        let output_conventional_commit_prefix_format = output_settings
            .conventional_commit_prefix_format
            .unwrap_or_default();
        let output_lang =
            Language::from_str(&output_settings.lang.unwrap_or_default()).unwrap_or_default();
        let output_show_per_file_summary = output_settings.show_per_file_summary.unwrap_or(false);
        Ok(Self {
            client: client.into(),
            prompt_file_diff,
            prompt_conventional_commit_prefix,
            prompt_commit_summary,
            prompt_commit_title,
            prompt_translation,
            output_lang,
            output_show_per_file_summary,
            output_conventional_commit,
            output_conventional_commit_prefix_format,
        })
    }

    pub(crate) async fn get_commit_message(&self, file_diffs: Vec<&str>) -> Result<String> {
        let mut set: JoinSet<Result<Option<(String, String)>>> = JoinSet::new();
        let mut summary_for_file = BTreeMap::new();

        for file_diff in file_diffs {
            if set.len() == 4 {
                if let Some((name, summary)) = set.join_next().await.unwrap()?? {
                    summary_for_file.insert(name, summary);
                }
            }
            let file_diff = file_diff.to_owned();
            let cloned_self = self.clone();
            set.spawn(async move { cloned_self.process_file_diff(&file_diff).await });
        }

        while let Some(res) = set.join_next().await {
            if let Some((k, v)) = res?? {
                summary_for_file.insert(k, v);
            }
        }

        if summary_for_file.is_empty() {
            bail!("No files to summarize.");
        }

        let summary_points = &summary_for_file
            .iter()
            .map(|(file_name, completion)| format!("[{file_name}]\n{completion}"))
            .collect::<Vec<String>>()
            .join("\n");

        let mut message = String::with_capacity(1024);

        let (title, completion, conventional_commit_prefix) = try_join!(
            self.commit_title(summary_points),
            self.commit_summary(summary_points),
            self.conventional_commit_prefix(summary_points)
        )?;

        message.push_str(&format!("{title}\n\n{completion}\n\n"));

        if self.output_show_per_file_summary {
            for (file_name, completion) in &summary_for_file {
                if !completion.is_empty() {
                    message.push_str(&format!("[{file_name}]\n{completion}\n"));
                }
            }
        }

        // split message into lines and uniquefy lines
        let mut lines = message.lines().collect::<Vec<&str>>();
        lines.dedup();
        let message = lines.join("\n");

        let mut message = self.commit_translate(&message).await?;
        if !conventional_commit_prefix.is_empty() {
            let mut ctx = Context::new();
            ctx.insert("prefix", conventional_commit_prefix.as_str());
            let formated_prefix =
                Tera::one_off(&self.output_conventional_commit_prefix_format, &ctx, false)?;
            message.insert_str(0, formated_prefix.as_str());
        }

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
    async fn process_file_diff(&self, file_diff: &str) -> Result<Option<(String, String)>> {
        if let Some(file_name) = util::get_file_name_from_diff(file_diff) {
            let completion = self
                .diff_summary(file_name, file_diff)
                .await
                .with_context(|| format!("Could not summarize {file_name}"))?;
            Ok(Some((file_name.to_string(), completion)))
        } else {
            Ok(None)
        }
    }

    async fn diff_summary(&self, file_name: &str, file_diff: &str) -> Result<String> {
        debug!("summarizing file: {}", file_name);

        let prompt = format_prompt(
            &self.prompt_file_diff,
            HashMap::from([("file_diff", file_diff)]),
        )?;

        self.client.completions(&prompt).await
    }

    // TODO use option type and enum here
    pub(crate) async fn conventional_commit_prefix(&self, summary_points: &str) -> Result<String> {
        if !self.output_conventional_commit {
            return Ok("".to_string());
        }
        let prompt = format_prompt(
            &self.prompt_conventional_commit_prefix,
            HashMap::from([("summary_points", summary_points)]),
        )?;

        let completion = self.client.completions(&prompt).await?;
        match completion.to_ascii_lowercase().trim() {
            "build" | "chore" | "ci" | "docs" | "feat" | "fix" | "perf" | "refactor" | "style"
            | "test" => Ok(completion.to_string()),
            _ => Ok("".to_string()),
        }
    }

    pub(crate) async fn commit_summary(&self, summary_points: &str) -> Result<String> {
        let prompt = format_prompt(
            &self.prompt_commit_summary,
            HashMap::from([("summary_points", summary_points)]),
        )?;

        self.client.completions(&prompt).await
    }

    pub(crate) async fn commit_title(&self, summary_points: &str) -> Result<String> {
        let prompt = format_prompt(
            &self.prompt_commit_title,
            HashMap::from([("summary_points", summary_points)]),
        )?;

        self.client.completions(&prompt).await
    }

    pub(crate) async fn commit_translate(&self, commit_message: &str) -> Result<String> {
        if let Language::En = self.output_lang {
            return Ok(commit_message.to_string());
        }
        let prompt = format_prompt(
            &self.prompt_translation,
            HashMap::from([
                ("commit_message", commit_message),
                ("output_language", &self.output_lang.to_string()),
            ]),
        )?;
        self.client.completions(&prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug, Default)]
    struct Calls {
        active: AtomicUsize,
        peak: AtomicUsize,
        total: AtomicUsize,
    }

    #[derive(Debug)]
    struct RecordingClient {
        calls: Arc<Calls>,
        fail: bool,
    }

    #[async_trait::async_trait]
    impl LlmClient for RecordingClient {
        async fn completions(&self, _prompt: &str) -> Result<String> {
            self.calls.total.fetch_add(1, Ordering::SeqCst);
            let active = self.calls.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.calls.peak.fetch_max(active, Ordering::SeqCst);
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            self.calls.active.fetch_sub(1, Ordering::SeqCst);
            if self.fail {
                bail!("test provider failure");
            }
            Ok("summary".to_string())
        }
    }

    #[tokio::test]
    async fn bounds_concurrency_and_stops_on_incomplete_summaries() {
        for fail in [false, true] {
            let calls = Arc::new(Calls::default());
            let client = SummarizationClient::new(
                Settings::default(),
                Box::new(RecordingClient {
                    calls: calls.clone(),
                    fail,
                }),
            )
            .unwrap();
            let diffs = (0..8)
                .map(|i| format!("diff --git a/file{i} b/file{i}\n"))
                .collect::<Vec<_>>();
            let result = client
                .get_commit_message(diffs.iter().map(String::as_str).collect())
                .await;
            assert_eq!(calls.peak.load(Ordering::SeqCst), 4);
            if fail {
                assert!(result
                    .unwrap_err()
                    .to_string()
                    .contains("Could not summarize file"));
                assert_eq!(calls.total.load(Ordering::SeqCst), 4);
            } else {
                assert!(result.is_ok());
                assert_eq!(calls.total.load(Ordering::SeqCst), 11);
            }
        }
    }

    #[tokio::test]
    async fn empty_input_makes_no_requests() {
        let calls = Arc::new(Calls::default());
        let client = SummarizationClient::new(
            Settings::default(),
            Box::new(RecordingClient {
                calls: calls.clone(),
                fail: false,
            }),
        )
        .unwrap();
        assert!(client.get_commit_message(vec![]).await.is_err());
        assert_eq!(calls.total.load(Ordering::SeqCst), 0);
    }
    fn filter<'a>(patterns: &[&str], diffs: Vec<&'a str>) -> Result<Vec<&'a str>> {
        filter_diffs(
            &patterns.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
            diffs,
        )
    }

    #[test]
    fn filters_diffs_using_git_style_patterns() {
        let patterns = &[
            "Cargo.lock",
            "/generated/",
            "**/*.min.js",
            "vendor/*",
            "!vendor/README.md",
        ];
        for (path, ignored) in [
            ("Cargo.lock", true),
            ("crates/tool/Cargo.lock", true),
            ("Cargo.lock.backup", false),
            ("generated/data.rs", true),
            ("src/generated/data.rs", false),
            ("app.min.js", true),
            ("assets/app.min.js", true),
            ("assets/app.js", false),
            ("vendor/library.rs", true),
            ("vendor/README.md", false),
        ] {
            let diff = format!("diff --git a/{path} b/{path}\n");
            assert_eq!(
                filter(patterns, vec![&diff]).unwrap().is_empty(),
                ignored,
                "{path}"
            );
        }
    }

    #[test]
    fn later_rules_override_earlier_rules() {
        let diff = "diff --git a/keep.lock b/keep.lock\n";
        assert_eq!(
            filter(&["*.lock", "!keep.lock"], vec![diff]).unwrap(),
            vec![diff]
        );
        assert!(filter(&["!keep.lock", "*.lock"], vec![diff])
            .unwrap()
            .is_empty());
        assert_eq!(filter(&[], vec![diff]).unwrap(), vec![diff]);
        assert!(filter(&[], vec![""]).unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_patterns_with_context() {
        let error = filter(&["[z-a]"], vec![]).unwrap_err().to_string();
        assert!(error.contains("Invalid file_ignore pattern: [z-a]"));
    }
}
