use anyhow::Result;
use clap::arg;
use clap::ValueEnum;
use colored::Colorize;

use clap::Args;

use std::fs;

use std::path::PathBuf;

use crate::git;

use crate::help::print_help_openai_api_key;
use crate::llms::{llm_client::LlmClient, openai::OpenAIClient};
use crate::settings::ModelProvider;

use crate::settings::Settings;
use crate::summarize::SummarizationClient;
use crate::util::SplitPrefixInclusive;

use crate::llms::tester_foobar::FooBarClient;

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

fn get_llm_client(settings: &Settings) -> Box<dyn LlmClient> {
    match settings {
        Settings {
            model_provider: Some(ModelProvider::TesterFoobar),
            ..
        } => Box::new(FooBarClient::new().unwrap()),
        Settings {
            model_provider: Some(ModelProvider::OpenAI),
            openai: Some(openai),
            ..
        } => {
            let client = OpenAIClient::new(openai.to_owned());
            if let Err(_e) = client {
                print_help_openai_api_key();
                panic!("OpenAI API key not found in config or environment");
            }
            Box::new(client.unwrap())
        }
        _ => panic!("Could not load LLM Client from config!"),
    }
}

pub(crate) async fn main(settings: Settings, args: PrepareCommitMsgArgs) -> Result<()> {
    match (args.commit_source, settings.allow_amend) {
        (CommitSource::Empty, _) | (CommitSource::Commit, Some(true)) => {}
        (CommitSource::Commit, _) => {
            println!("🤖 Skipping gptcommit because commit is being amended");
            return Ok(());
        }
        _ => {
            println!("🤖 Skipping gptcommit because githook is not run on commit");
            return Ok(());
        }
    };

    let client = get_llm_client(&settings);
    let summarization_client = SummarizationClient::new(settings.to_owned(), client)?;

    println!("{}", "🤖 Asking GPT-3 to summarize diffs...".green().bold());

    let output = if let Some(git_diff_output) = args.git_diff_content {
        fs::read_to_string(git_diff_output)?
    } else {
        git::get_diffs()?
    };

    let file_diffs = output.split_prefix_inclusive("\ndiff --git ");
    let commit_message = summarization_client.get_commit_message(file_diffs).await?;

    // prepend output to commit message
    let mut original_message: String = if args.commit_msg_file.is_file() {
        fs::read_to_string(&args.commit_msg_file)?
    } else {
        String::new()
    };
    if settings.allow_amend.unwrap_or(false) {
        original_message = original_message
            .lines()
            .map(|l| format!("# {l}"))
            .collect::<Vec<String>>()
            .join("\n");
        original_message = format!("### BEGIN GIT COMMIT BEFORE AMEND\n{original_message}\n### END GIT COMMIT BEFORE AMEND\n");
    }
    fs::write(
        &args.commit_msg_file,
        format!("{commit_message}\n{original_message}"),
    )?;

    Ok(())
}
