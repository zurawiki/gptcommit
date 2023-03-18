use actions::{config::ConfigArgs, prepare_commit_msg::PrepareCommitMsgArgs};

#[macro_use]
extern crate log;
mod cmd;
mod git;
mod prompt;
mod summarize;
mod toml;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod actions;
mod help;
mod llms;
mod settings;

use log::LevelFilter;
use settings::Settings;
use simple_logger::SimpleLogger;

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
    /// Uninstall the git hook
    Uninstall,
    /// Read and modify settings
    Config(ConfigArgs),
    /// Run on the prepare-commit-msg hook
    PrepareCommitMsg(PrepareCommitMsgArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    debug!("CLI args: {:?}", cli);

    SimpleLogger::new()
        .with_level(if cli.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Warn
        })
        .env()
        .init()?;

    let settings = Settings::new()?;
    debug!("Settings: {:?}", settings);

    match cli.action {
        Action::Config(cli) => actions::config::main(settings, cli).await,
        Action::Install => actions::install::main(settings).await,
        Action::Uninstall => actions::uninstall::main(settings).await,
        Action::PrepareCommitMsg(cli) => actions::prepare_commit_msg::main(settings, cli).await,
    }
}
