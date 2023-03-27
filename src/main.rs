#[macro_use]
extern crate log;

mod actions;
pub mod cli;
mod cmd;
mod git;
mod help;
mod llms;
mod prompt;
mod settings;
mod summarize;
mod toml;
mod util;

use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use settings::Settings;
use simple_logger::SimpleLogger;

use crate::cli::Action;

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = cli::GptcommitCLI::parse();
    SimpleLogger::new()
        .with_level(if cli_args.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Warn
        })
        .env()
        .init()?;
    debug!("gptcommit v{}", env!("CARGO_PKG_VERSION"));

    debug!("CLI args: {:?}", cli_args);

    let settings = Settings::new()?;
    debug!("Settings: {:?}", settings);

    match cli_args.action {
        Action::Config(cli_args) => actions::config::main(settings, cli_args).await,
        Action::Install => actions::install::main(settings).await,
        Action::Uninstall => actions::uninstall::main(settings).await,
        Action::PrepareCommitMsg(cli_args) => {
            actions::prepare_commit_msg::main(settings, cli_args).await
        }
    }
}
