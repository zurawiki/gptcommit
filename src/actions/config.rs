use std::{collections::VecDeque, fs};

use clap::{Args, Subcommand};
use toml::Value;

use crate::settings::{get_user_config_path, Settings};
use anyhow::{Result, bail};

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigAction {
    /// List all config values
    List {
        /// if set, will save the config to the user's config file
        #[clap(long)]
        save: bool,
    },
    /// Read config value
    Get { key: String },
    /// Set config value
    Set { key: String, value: String },
    /// Clear config value
    Delete { key: String },
}

#[derive(Args, Debug)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    action: ConfigAction,
}

pub(crate) async fn main(settings: Settings, args: ConfigArgs) -> Result<()> {
    debug!("Config subcommand - Settings = {:?}", settings);

    match args.action {
        ConfigAction::List { save } => list(settings, save).await,
        ConfigAction::Get { key } => get(settings, key).await,
        ConfigAction::Set { key, value } => set(settings, key, value).await,
        ConfigAction::Delete { key } => delete(settings, key).await,
    }
}

async fn delete(settings: Settings, full_key: String) -> Result<()> {
    let user_config_path = get_user_config_path().expect("Could not find user config path");
    let toml_string = toml::to_string_pretty(&settings).unwrap();

    let mut root: Value = toml::from_str(&toml_string)?;

    let mut node: &mut Value = &mut root;
    let mut path = key_to_path(&full_key);

    // Find the node that we want to set the value on
    while path.len() > 1 {
        let key = path.get(0).unwrap();
        if let Some(child_config) = node.get_mut(key) {
            node = child_config;
            path.pop_front();
        } else {
            bail!("Config key {} not found", &full_key);
        }
    }

    if path.len() == 1 {
        let last_key = path.get(0).unwrap();
        if node.get_mut(last_key).unwrap().is_table() || node.get_mut(last_key).unwrap().is_array()
        {
            bail!(
                "Config key {} is a table or array, cannot clear value",
                &full_key
            );
        }
        //set node value to value
        node.as_table_mut().unwrap().remove(path.get(0).unwrap());
        // write out new root toml
        let new_toml_str = toml::to_string_pretty(&root).unwrap();
        // write out new root toml
        fs::write(user_config_path, new_toml_str)?;
        println!("Cleared {}", full_key);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Config key {} not found", full_key))
    }
}

async fn set(settings: Settings, full_key: String, value: String) -> Result<()> {
    let user_config_path = get_user_config_path().expect("Could not find user config path");
    let toml_string = toml::to_string_pretty(&settings).unwrap();

    let mut root: Value = toml::from_str(&toml_string)?;

    let mut node: &mut Value = &mut root;
    let mut path = key_to_path(&full_key);

    // Find the node that we want to set the value on
    while path.len() > 1 {
        let key = path.get(0).unwrap();
        if let Some(child_config) = node.get_mut(key) {
            node = child_config;
            path.pop_front();
        } else {
            bail!("Config key {} not found", &full_key);
        }
    }

    if path.len() == 1 {
        let last_key = path.get(0).unwrap();
        if node.get_mut(last_key).unwrap().is_table() || node.get_mut(last_key).unwrap().is_array()
        {
            bail!(
                "Config key {} is a table or array, cannot set value",
                &full_key
            );
        }
        //set node value to value
        node.as_table_mut().unwrap().insert(
            path.get(0).unwrap().to_owned(),
            toml::Value::String(value.to_owned()),
        );
        // write out new root toml
        let new_toml_str = toml::to_string_pretty(&root).unwrap();
        // write out new root toml
        fs::write(user_config_path, new_toml_str)?;
        println!("{} = {}", full_key, value);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Config key {} not found", full_key))
    }
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
    println!("{}", toml_string);
    if save {
        let user_config_path = get_user_config_path().expect("Could not find user config path");
        fs::write(&user_config_path, toml_string)?;
        println!("Config saved to {}", user_config_path.display());
    }
    Ok(())
}
