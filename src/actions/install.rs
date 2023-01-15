use std::{
    fs::{self},
    process::Command,
    path::PathBuf,
};

#[cfg(unix)]
use std::{
    fs::{self, Permissions},
    os::unix::prelude::PermissionsExt,
};


use ansi_term::Colour;
use anyhow::{anyhow, bail, Result};
use which::which;

fn find_executable(name: &str, error_msg: &str) -> Result<PathBuf> {
    let path = which(name).map_err(|_| {
        anyhow!(
            "Could not find `{}` executable in PATH. {}",
            name,
            error_msg
        )
    })?;
    info!("Found {} executable at {:?}", name, path);

    Ok(path)
}

pub(crate) async fn main() -> Result<()> {
    println!(
        "{}",
        Colour::Green.bold().paint("Installing gptcommit hook...")
    );

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
        Colour::Green
            .bold()
            .paint("gptcommit hook successfully installed!"),
        Colour::Yellow
            .bold()
            .paint("Make sure to set OPENAI_API_KEY when using `git commit`.")
    );

    Ok(())
}

/// Given current working directory, return path to .git/hooks
fn get_hooks_path() -> Result<PathBuf> {
    let command_output = Command::new("git")
        .args(["rev-parse", "--show-toplevel", "--git-path", "hooks"])
        .output()?;
    if !command_output.status.success() {
        let stderr = String::from_utf8_lossy(&command_output.stderr);
        bail!("{}", stderr);
    }

    let stdout = String::from_utf8(command_output.stdout).expect("Invalid UTF-8");
    let rel_hooks_path = stdout.lines().last().unwrap().to_string();
    // turn relative path into absolute path
    let hooks_path = std::fs::canonicalize(rel_hooks_path)?;
    debug!("Found hooks path: {:?}", hooks_path);
    Ok(hooks_path)
}
