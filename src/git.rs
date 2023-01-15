use crate::cmd;
use anyhow::Result;

pub(crate) fn get_diffs() -> Result<String> {
    let output = cmd::run_command("git", &["diff", "--staged"])?;

    Ok(output)
}
