use ansi_term::Colour;
use anyhow::Result;
use clap::arg;

use clap::Args;

use std::collections::HashMap;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tokio::task::JoinSet;

use crate::git;
use crate::summarize;
use crate::util;

async fn process_file_diff(file_diff: &str) -> Option<(String, String)> {
    if let Some(file_name) = util::get_file_name_from_diff(file_diff) {
        let completion = summarize::diff_summary(file_name, file_diff).await;
        Some((
            file_name.to_string(),
            completion.unwrap_or_else(|_| "".to_string()),
        ))
    } else {
        None
    }
}

#[derive(Args, Debug)]
pub(crate) struct PrepareCommitMsgArgs {
    /// Name of the file that has the commit message
    #[arg(long)]
    commit_msg_file: PathBuf,

    /// Description of the commit message's source
    #[arg(long)]
    commit_source: Option<String>,

    /// SHA1 hash of the commit being amended
    #[arg(long)]
    commit_sha: Option<String>,

    /// Debugging tool to mock git repo state
    #[arg(long)]
    git_diff_content: Option<PathBuf>,
}

pub(crate) async fn main(args: PrepareCommitMsgArgs) -> Result<()> {
    println!(
        "{}",
        Colour::Green
            .bold()
            .paint("🤖 Asking GPT-3 to summarize diffs...")
    );

    let output = if let Some(git_diff_output) = args.git_diff_content {
        fs::read_to_string(git_diff_output)?
    } else {
        git::get_diffs()?
    };

    let file_diffs = util::split_prefix_inclusive(&output, "\ndiff --git ");

    let mut set = JoinSet::new();

    for file_diff in file_diffs {
        let file_diff = file_diff.to_owned();
        set.spawn(async move { process_file_diff(&file_diff).await });
    }

    let mut summary_for_file: HashMap<String, String> = HashMap::with_capacity(set.len());
    while let Some(res) = set.join_next().await {
        if let Some((k, v)) = res.unwrap() {
            summary_for_file.insert(k, v);
        }
    }

    let summary_points = &summary_for_file
        .iter()
        .map(|(file_name, completion)| format!("[{}]\n{}", file_name, completion))
        .collect::<Vec<String>>()
        .join("\n");

    let completion = summarize::commit_summary(summary_points).await?;

    // overwrite commit message file
    let mut commit_msg_path = File::create(args.commit_msg_file)?;

    writeln!(commit_msg_path, "# AI Generated Commit Summary:")?;
    writeln!(commit_msg_path, "# EDIT BELOW AS NEEDED")?;
    writeln!(commit_msg_path, "{}", completion)?;
    writeln!(commit_msg_path, "#")?;
    writeln!(commit_msg_path, "# File-level changes:")?;
    for (file_name, completion) in &summary_for_file {
        if !completion.is_empty() {
            writeln!(commit_msg_path, "[{}]", file_name)?;
            writeln!(commit_msg_path, "{}", completion)?;
        }
    }

    Ok(())
}
