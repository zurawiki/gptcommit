use std::fs::{self};

use anyhow::Result;
use colored::Colorize;

use crate::{cmd::find_executable, git::get_hooks_path, settings::Settings};

pub(crate) async fn main(_settings: Settings) -> Result<()> {
    println!("{}", "Uninstalling gptcommit hook...".green().bold());

    find_executable("git", "To use gptcommit, you must have git on your PATH")?;
    find_executable("gptcommit", " To use gptcommit, you must have gptcommit on your PATH. Install with `cargo install --locked gptcommit`")?;

    // confirm in git root
    let hooks_path = get_hooks_path()?;
    info!(
        "Found git hooks path for current git repo {}",
        hooks_path.display()
    );
    let prepare_commit_msg_path = hooks_path.join("prepare-commit-msg");
    info!("Removing file at {}", prepare_commit_msg_path.display());

    if prepare_commit_msg_path.exists() {
        let file_contents = fs::read_to_string(&prepare_commit_msg_path)?;
        if file_contents == include_str!("../../prepare-commit-msg") {
            fs::remove_file(&prepare_commit_msg_path)?;
            println!(
                "{}",
                "gptcommit hook successfully uninstalled!".green().bold(),
            );
        } else {
            warn!(
                "{} is not gptcommit's prepare-commit-msg hook. Skipping uninstall.",
                prepare_commit_msg_path.display()
            );
            warn!(
                "Manually delete this file with\n  rm -rf {}",
                prepare_commit_msg_path.display()
            );
        }
    }

    Ok(())
}
