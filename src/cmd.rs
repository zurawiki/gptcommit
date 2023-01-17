use anyhow::bail;
use anyhow::Result;
use std::process::Command;

pub(crate) fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{}", stderr);
    }

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    Ok(stdout)
}
