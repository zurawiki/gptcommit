use crate::actions::{config::ConfigArgs, prepare_commit_msg::PrepareCommitMsgArgs};
use clap::{Parser, Subcommand};

/// Represents the main command-line interface for the application.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct GptcommitCLI {
    /// The action to perform (subcommand).
    #[command(subcommand)]
    pub action: Option<Action>,
    /// Enable verbose logging.
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// Actions the application can perform.
#[derive(Subcommand, Debug)]
pub(crate) enum Action {
    /// Install the git hook
    Install,
    /// Uninstall the git hook
    Uninstall,
    /// Read and modify settings
    Config(ConfigArgs),
    /// Run on the prepare-commit-msg hook
    PrepareCommitMsg(PrepareCommitMsgArgs),
}
