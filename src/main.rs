use actions::prepare_commit_msg::PrepareCommitMsgArgs;
use log::LevelFilter;
use simple_logger::SimpleLogger;
#[macro_use]
extern crate log;
mod cmd;
mod git;
mod openai;
mod summarize;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod actions;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    action: Action,
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Install the git hook
    Install,
    /// Run on the prepare-commit-msg hook
    PrepareCommitMsg(PrepareCommitMsgArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    debug!("{:?}", cli);

    // Remove dates from logger
    SimpleLogger::new()
        .with_level(if cli.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Warn
        })
        .env()
        .init()?;

    match cli.action {
        Action::Install => actions::install::main().await,
        Action::PrepareCommitMsg(cli) => actions::prepare_commit_msg::main(cli).await,
    }
}
