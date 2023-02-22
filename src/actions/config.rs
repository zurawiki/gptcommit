use std::{collections::VecDeque, fs, path::PathBuf};

use clap::{Args, Subcommand};
use toml::Value;

use crate::{
    settings::{get_local_config_path, get_user_config_path, Settings},
    toml::DeepKeysCollector,
};
use anyhow::{bail, Result};

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigAction {
    /// List all config keys
    Keys,
    /// List all config values
    List {
        /// if set, will save the config to the user's config file
        #[clap(long)]
        save: bool,
    },
    /// Read config value
    Get { key: String },
    /// Set config value
    Set {
        key: String,
        value: String,
        /// if set, modifies the local config. Default behavior modifies global config
        #[clap(long)]
        local: bool,
    },
    /// Clear config value
    Delete {
        key: String,
        /// if set, modifies the local config. Default behavior modifies global config
        #[clap(long)]
        local: bool,
    },
}

#[derive(Args, Debug)]
pub(crate) struct ConfigArgs {
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
            bail!("No repo-local config found. Please run `git init` to create a repo first");
        }
    } else if let Some(config_path) = get_user_config_path() {
        Ok(config_path)
    } else {
        bail!("No user config found.");
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
    let settings = &Settings::from_clear(&full_key)?;
    let toml_string = toml::to_string_pretty(settings).unwrap();
    let config_path = get_config_path(local)?;
    fs::write(&config_path, toml_string)?;
    println!("Cleared {full_key}");
    println!("Config saved to {}", config_path.display());
    Ok(())
}

async fn set(_settings: Settings, full_key: String, value: String, local: bool) -> Result<()> {
    let settings = &Settings::from_set_override(&full_key, &value)?;
    let toml_string = toml::to_string_pretty(settings).unwrap();
    let config_path = get_config_path(local)?;
    fs::write(&config_path, toml_string)?;
    println!("{full_key} = {value}");
    println!("Config saved to {}", config_path.display());
    Ok(())
}

fn key_to_path(key: &str) -> VecDeque<String> {
    key.split('.').map(|s| s.to_string()).collect()
}

async fn get(settings: Settings, full_key: String) -> Result<()> {
    let toml_string = toml::to_string_pretty(&settings).unwrap();

    let mut node: &Value = &toml::from_str(&toml_string)?;
    let mut path = key_to_path(&full_key);
    while let Some(key) = path.get(0) {
        if let Some(child_config) = node.get(key) {
            node = child_config;
            path.pop_front();
        } else {
            bail!("Config key {} not found", full_key);
        }
    }

    if path.is_empty() {
        println!("{}", node.as_str().unwrap_or(""));
    } else {
        bail!("Config key {} not found", full_key);
    }
    Ok(())
}

async fn list(settings: Settings, save: bool) -> Result<()> {
    let toml_string = toml::to_string_pretty(&settings).unwrap();
    println!("{toml_string}");
    if save {
        let user_config_path = get_user_config_path().expect("Could not find user config path");
        fs::write(&user_config_path, toml_string)?;
        println!("Config saved to {}", user_config_path.display());
    }
    Ok(())
}
