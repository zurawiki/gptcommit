use anyhow::bail;
use anyhow::Result;
use regex::Regex;

use std::collections::{HashMap, HashSet};

pub fn format_prompt(prompt: &str, map: HashMap<&str, &str>) -> Result<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new("<([A-Z_]+)>").unwrap();
    }

    let required_keys: HashSet<String> = RE
        .captures_iter(prompt)
        .map(|cap| cap[1].to_string())
        .collect();
    let provided_keys: HashSet<String> = map.keys().map(|s| s.to_string()).collect();

    if !required_keys.eq(&provided_keys) {
        bail!(
            r#"Required keys did not match provided keys.
  Required: {:?}
  Provided: {:?}
  Prompt: {}"#,
            required_keys,
            provided_keys,
            prompt
        );
    }

    let mut result = prompt.to_string();
    for (key, value) in map {
        result = result.replace(&format!("<{key}>"), value);
    }
    Ok(result)
}

pub static PROMPT_TO_SUMMARIZE_DIFF: &str =
    include_str!("../prompts/summarize_file_diff.prompt.txt");
pub static PROMPT_TO_SUMMARIZE_DIFF_SUMMARIES: &str =
    include_str!("../prompts/summarize_commit.prompt.txt");
pub static PROMPT_TO_SUMMARIZE_DIFF_TITLE: &str =
    include_str!("../prompts/title_commit.prompt.txt");
