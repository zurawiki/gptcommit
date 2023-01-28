use std::{
    fs::{self},
    path::PathBuf,
    process::Command,
};

#[cfg(unix)]
use std::{fs::Permissions, os::unix::prelude::PermissionsExt};

use crate::cmd;
use anyhow::{bail, Result};
use colored::Colorize;

pub(crate) fn get_diffs() -> Result<String> {
    let output = cmd::run_command("git", &["diff", "--staged", "-w"])?;

    Ok(output)
}

/// Given current working directory, return path to .git/hooks
pub(crate) fn get_hooks_path() -> Result<PathBuf> {
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
