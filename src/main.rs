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
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    Install,
    PrepareCommitMsg(PrepareCommitMsgArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Remove dates from logger
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .env()
        .init()?;

    let args = Args::parse();
    debug!("{:?}", args);
    match args.action {
        Action::Install => actions::install::main().await,
        Action::PrepareCommitMsg(args) => actions::prepare_commit_msg::main(args).await,
    }
}
