use std::{collections::VecDeque, fs, path::PathBuf};

use clap::{Args, Subcommand};
use toml::Value;

use crate::{
    settings::{get_local_config_path, get_user_config_path, Settings},
    toml::DeepKeysCollector,
};
use anyhow::{anyhow, bail, Context, Result};
use toml_edit::{DocumentMut, Item, Table};

/// Actions related to application configuration.
#[derive(Subcommand, Debug)]
pub(crate) enum ConfigAction {
    /// List all config keys
    Keys,
    /// List all config values
    List {
        /// If set, will save the config to the user's config file
        #[clap(long)]
        save: bool,
    },
    /// Read a config value
    Get { key: String },
    /// Set a config value
    Set {
        key: String,
        value: String,
        /// If set, modifies the local config. Default behavior modifies global config
        #[clap(long)]
        local: bool,
    },
    /// Clear a config value
    Delete {
        key: String,
        /// If set, modifies the local config. Default behavior modifies global config
        #[clap(long)]
        local: bool,
    },
}

/// Configuration-related command-line arguments
#[derive(Args, Debug)]
pub(crate) struct ConfigArgs {
    /// The action to perform (subcommand)
    #[command(subcommand)]
    action: ConfigAction,
}

pub(crate) async fn main(settings: Settings, args: ConfigArgs) -> Result<()> {
    debug!("Config subcommand - Settings = {:?}", settings);

    match args.action {
        ConfigAction::Keys => keys(settings).await,
        ConfigAction::List { save } => list(settings, save).await,
        ConfigAction::Get { key } => get(settings, key).await,
        ConfigAction::Set { key, value, local } => set(settings, key, value, local).await,
        ConfigAction::Delete { key, local } => delete(settings, key, local).await,
    }
}

fn get_config_path(local: bool) -> Result<PathBuf> {
    if local {
        if let Some(config_path) = get_local_config_path() {
            Ok(config_path)
        } else {
            bail!("No local repository configuration found. Please run `git init` to create a repository first.");
        }
    } else if let Some(config_path) = get_user_config_path() {
        Ok(config_path)
    } else {
        bail!("No user configuration found.");
    }
}

async fn keys(settings: Settings) -> Result<()> {
    let toml_string = toml::to_string_pretty(&settings).unwrap();
    let keys = DeepKeysCollector::get_keys(toml_string);
    for key in keys {
        println!("{key}");
    }
    Ok(())
}

async fn delete(_settings: Settings, full_key: String, local: bool) -> Result<()> {
    let config_path = get_config_path(local)?;
    update_file(&config_path, &full_key, None)?;
    println!("Cleared {full_key}");
    println!("Config saved to {}", config_path.display());
    Ok(())
}

async fn set(_settings: Settings, full_key: String, value: String, local: bool) -> Result<()> {
    let value_item = parse_setting(&full_key, &value)?;
    let config_path = get_config_path(local)?;
    update_file(&config_path, &full_key, Some(value_item))?;
    println!("{full_key} = {value}");
    println!("Config saved to {}", config_path.display());
    Ok(())
}

fn parse_setting(key: &str, value: &str) -> Result<Item> {
    let settings = Settings::from_set_override(key, value)?;
    let document: DocumentMut = toml::to_string(&settings)?.parse()?;
    let mut item = document.as_item();
    for part in key.split('.') {
        item = item
            .get(part)
            .ok_or_else(|| anyhow!("Unknown configuration key '{key}'."))?;
    }
    if !item.is_value() {
        bail!("Configuration key '{key}' must name a value, not a section.");
    }
    Ok(item.clone())
}

fn update_file(path: &std::path::Path, key: &str, value: Option<Item>) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Could not read config at {}", path.display()))?;
    let updated = update_document(&source, key, value)?;
    fs::write(path, updated)
        .with_context(|| format!("Could not write config at {}", path.display()))
}

fn update_document(source: &str, key: &str, value: Option<Item>) -> Result<String> {
    let mut document: DocumentMut = source.parse()?;
    let parts: Vec<_> = key.split('.').collect();
    if parts.iter().any(|part| part.is_empty()) {
        bail!("Invalid configuration key '{key}'.");
    }
    let mut table = document.as_table_mut() as &mut dyn toml_edit::TableLike;
    for part in &parts[..parts.len() - 1] {
        if !table.contains_key(part) {
            if value.is_none() {
                return Ok(source.to_string());
            }
            table.insert(part, Item::Table(Table::new()));
        }
        table = table
            .get_mut(part)
            .and_then(Item::as_table_like_mut)
            .ok_or_else(|| anyhow!("'{part}' is not a configuration section."))?;
    }
    let leaf = parts[parts.len() - 1];
    if let Some(mut value) = value {
        if let (Some(old), Some(new)) = (
            table.get(leaf).and_then(Item::as_value),
            value.as_value_mut(),
        ) {
            *new.decor_mut() = old.decor().clone();
        }
        table.insert(leaf, value);
    } else {
        table.remove(leaf);
    }
    Ok(document.to_string())
}

fn key_to_path(key: &str) -> VecDeque<String> {
    key.split('.').map(|s| s.to_string()).collect()
}

async fn get(settings: Settings, full_key: String) -> Result<()> {
    let toml_string = toml::to_string_pretty(&settings).unwrap();

    let mut node: &Value = &toml::from_str(&toml_string)?;
    let mut path = key_to_path(&full_key);
    while let Some(key) = path.front() {
        if let Some(child_config) = node.get(key) {
            node = child_config;
            path.pop_front();
        } else {
            bail!("Configuration key '{}' not found.", full_key);
        }
    }

    if path.is_empty() {
        println!("{}", display_value(node));
    } else {
        bail!("Configuration key '{}' not found.", full_key);
    }
    Ok(())
}

fn display_value(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}

async fn list(settings: Settings, save: bool) -> Result<()> {
    let toml_string = toml::to_string_pretty(&settings).unwrap();
    println!("{toml_string}");
    if save {
        let user_config_path =
            get_user_config_path().expect("Could not find user configuration path");
        fs::write(&user_config_path, toml_string)?;
        println!("Config saved to {}", user_config_path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_non_string_values() {
        assert_eq!(display_value(&Value::Boolean(false)), "false");
        assert_eq!(display_value(&Value::Integer(2)), "2");
        assert_eq!(display_value(&Value::String("model".into())), "model");
        assert_eq!(
            display_value(&Value::Array(vec![Value::String("*.lock".into())])),
            "[\"*.lock\"]"
        );
    }

    #[test]
    fn sets_only_the_requested_key_and_preserves_comments() {
        let source =
            "# my preferences\n[openai]\nmodel = 'old' # keep this note\n\n[custom]\nvalue = 7\n";
        let updated = update_document(
            source,
            "openai.model",
            Some(parse_setting("openai.model", "new").unwrap()),
        )
        .unwrap();
        assert_eq!(
            updated,
            "# my preferences\n[openai]\nmodel = \"new\" # keep this note\n\n[custom]\nvalue = 7\n"
        );
        assert!(!updated.contains("api_key"));
        assert!(!updated.contains("prompt"));
    }

    #[test]
    fn creates_sparse_typed_overrides() {
        for (key, input, expected) in [
            ("allow_amend", "true", Value::Boolean(true)),
            ("openai.retries", "3", Value::Integer(3)),
            ("openai.model", "123", Value::String("123".into())),
            (
                "file_ignore",
                "[\"*.lock\", \"vendor/\"]",
                Value::Array(vec![
                    Value::String("*.lock".into()),
                    Value::String("vendor/".into()),
                ]),
            ),
        ] {
            let output =
                update_document("", key, Some(parse_setting(key, input).unwrap())).unwrap();
            let parsed: Value = output.parse().unwrap();
            let actual = key.split('.').fold(&parsed, |value, part| &value[part]);
            assert_eq!(actual, &expected);
            assert_eq!(parsed.as_table().unwrap().len(), 1);
        }
    }

    #[test]
    fn deletion_removes_only_the_selected_override() {
        let source = "# preferences\n[openai]\nmodel = 'local'\nretries = 4\n";
        assert_eq!(
            update_document(source, "openai.model", None).unwrap(),
            "# preferences\n[openai]\nretries = 4\n"
        );
        assert_eq!(
            update_document(source, "output.lang", None).unwrap(),
            source
        );
    }

    #[test]
    fn rejects_unknown_keys_and_invalid_values() {
        for (key, value) in [
            ("openai.modle", "x"),
            ("openai.retries", "nope"),
            ("allow_amend", "nope"),
            ("output.lang", "invalid"),
            ("file_ignore", "[1]"),
        ] {
            assert!(parse_setting(key, value).is_err(), "accepted {key}={value}");
        }
        assert!(update_document(
            "openai = 1\n",
            "openai.model",
            Some(parse_setting("openai.model", "x").unwrap())
        )
        .is_err());
    }
}
