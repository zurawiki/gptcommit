use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use which::which;
/// Runs the command with the given arguments and returns its stdout if the command
/// exits successfully. If the command fails, returns an error.
pub(crate) fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{}", stderr);
    }

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    Ok(stdout)
}

pub(crate) fn find_executable(name: &str, error_msg: &str) -> Result<PathBuf> {
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
