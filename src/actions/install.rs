use std::{
    fs::{self},
    path::PathBuf,
    process::Command,
};

#[cfg(unix)]
use std::{fs::Permissions, os::unix::prelude::PermissionsExt};

use anyhow::{bail, Result};
use colored::Colorize;

use crate::cmd::find_executable;

pub(crate) async fn main() -> Result<()> {
    println!("{}", "Installing gptcommit hook...".green().bold());

    find_executable("git", "To use gptcommit, you must have git on your PATH")?;
    find_executable("gptcommit", " To use gptcommit, you must have gptcommit on your PATH. Install with `cargo install gptcommit`")?;

    // confirm in git root
    let hooks_path = get_hooks_path()?;
    info!(
        "Found git hooks path for current git repo {}",
        hooks_path.display()
    );
    let prepare_commit_msg_path = hooks_path.join("prepare-commit-msg");
    info!("Overwriting file at {}", prepare_commit_msg_path.display());
    fs::write(
        &prepare_commit_msg_path,
        include_str!("../../prepare-commit-msg"),
    )?;
    #[cfg(unix)]
    fs::set_permissions(&prepare_commit_msg_path, Permissions::from_mode(0o755))?;

    println!(
        "{}\n{}",
        "gptcommit hook successfully installed!".green().bold(),
        "Make sure to set OPENAI_API_KEY when using `git commit`."
            .yellow()
            .bold()
    );

    Ok(())
}

/// Given current working directory, return path to .git/hooks
fn get_hooks_path() -> Result<PathBuf> {
    let command_output = Command::new("git")
        .args(["rev-parse", "--show-toplevel", "--git-path", "hooks"])
        .output()?;
    info!("Repo path from git: {:?}", command_output);

    if !command_output.status.success() {
        let stderr = String::from_utf8_lossy(&command_output.stderr);
        bail!("{}", stderr);
    }

    let stdout = String::from_utf8(command_output.stdout).expect("Invalid UTF-8");
    let rel_hooks_path = stdout.lines().last().unwrap().to_string();
    info!("Creating dir at {}", rel_hooks_path);
    // create dirs first otherwise canonicalize will fail
    fs::create_dir_all(&rel_hooks_path)?;
    #[cfg(unix)]
    fs::set_permissions(&rel_hooks_path, Permissions::from_mode(0o755))?;
    // turn relative path into absolute path
    let hooks_path = std::fs::canonicalize(rel_hooks_path)?;
    println!(
        "Installing git hook to {}",
        hooks_path.display().to_string().bold()
    );

    Ok(hooks_path)
}
