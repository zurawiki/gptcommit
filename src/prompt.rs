use tera::{Context, Error};

use std::collections::HashMap;
use tera::Tera;

pub fn format_prompt(prompt: &str, map: HashMap<&str, &str>) -> Result<String, Error> {
    let context = Context::from_serialize(map)?;

    Tera::one_off(prompt, &context, false)
}

pub static PROMPT_TO_CONVENTIONAL_COMMIT_PREFIX: &str =
    include_str!("../prompts/conventional_commit.tera");
pub static PROMPT_TO_SUMMARIZE_DIFF: &str = include_str!("../prompts/summarize_file_diff.tera");
pub static PROMPT_TO_SUMMARIZE_DIFF_SUMMARIES: &str =
    include_str!("../prompts/summarize_commit.tera");
pub static PROMPT_TO_SUMMARIZE_DIFF_TITLE: &str = include_str!("../prompts/title_commit.tera");
pub static PROMPT_TO_TRANSLATE: &str = include_str!("../prompts/translation.tera");
