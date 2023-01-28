use anyhow::bail;
use anyhow::Result;
use clap::arg;
use clap::ValueEnum;
use colored::Colorize;

use clap::Args;
use tokio::try_join;

use std::collections::HashMap;

use std::fs;

use std::path::PathBuf;
use tokio::task::JoinSet;

use crate::git;

use crate::help::print_help_openai_api_key;
use crate::openai::OpenAIClient;

use crate::settings::Settings;
use crate::summarize::SummarizationClient;
use crate::util;

/// Splits the contents of a git diff by file.
///
/// The file path is the first string in the returned tuple, and the
/// file content is the second string in the returned tuple.
///
/// The function assumes that the file_diff input is well-formed
/// according to the Diff format described in the Git documentation:
/// https://git-scm.com/docs/git-diff
async fn process_file_diff(
    summarize_client: SummarizationClient,
    file_diff: &str,
) -> Option<(String, String)> {
    if let Some(file_name) = util::get_file_name_from_diff(file_diff) {
        let completion = summarize_client.diff_summary(file_name, file_diff).await;
        Some((
            file_name.to_string(),
            completion.unwrap_or_else(|_| "".to_string()),
        ))
    } else {
        None
    }
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum CommitSource {
    #[clap(name = "")]
    Empty,
    Message,
    Template,
    Merge,
    Squash,
    Commit,
}
impl Default for CommitSource {
    fn default() -> Self {
        CommitSource::Empty
    }
}

#[derive(Args, Debug)]
pub(crate) struct PrepareCommitMsgArgs {
    /// Name of the file that has the commit message
    #[arg(long)]
    commit_msg_file: PathBuf,

    /// Description of the commit message's source
    #[arg(long, value_enum)]
    commit_source: CommitSource,

    /// SHA1 hash of the commit being amended
    #[arg(long)]
    commit_sha: Option<String>,

    /// Debugging tool to mock git repo state
    #[arg(long)]
    git_diff_content: Option<PathBuf>,
}

async fn get_commit_message(client: SummarizationClient, diff_as_input: &str) -> Result<String> {
    let file_diffs = util::split_prefix_inclusive(diff_as_input, "\ndiff --git ");

    let mut set = JoinSet::new();

    for file_diff in file_diffs {
        let file_diff = file_diff.to_owned();
        let summarize_client = client.to_owned();
        set.spawn(async move { process_file_diff(summarize_client, &file_diff).await });
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
        client.commit_title(summary_points),
        client.commit_summary(summary_points)
    )?;

    let mut message = String::with_capacity(1024);

    message.push_str(&format!("{title}\n\n{completion}\n"));
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
pub(crate) async fn main(settings: Settings, args: PrepareCommitMsgArgs) -> Result<()> {
    match args.commit_source {
        CommitSource::Empty => {}
        CommitSource::Commit => {
            println!("🤖 Skipping gptcommit because commit is being amended");
            return Ok(());
        }
        _ => {
            println!("🤖 Skipping gptcommit because githook is not run on commit");
            return Ok(());
        }
    };

    let client = match OpenAIClient::new(settings.openai.unwrap_or_default()) {
        Ok(client) => client,
        Err(_e) => {
            print_help_openai_api_key();
            bail!("OpenAI API key not found in config or environment");
        }
    };
    let summarization_client = SummarizationClient::new(settings.prompt.unwrap(), client)?;

    println!("{}", "🤖 Asking GPT-3 to summarize diffs...".green().bold());

    let output = if let Some(git_diff_output) = args.git_diff_content {
        fs::read_to_string(git_diff_output)?
    } else {
        git::get_diffs()?
    };

    let commit_message = get_commit_message(summarization_client, &output).await?;

    // prepend output to commit message
    let original_message = fs::read_to_string(&args.commit_msg_file)?;
    fs::write(
        &args.commit_msg_file,
        format!("{commit_message}\n{original_message}"),
    )?;

    Ok(())
}
