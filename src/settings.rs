use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};

use config::{
    builder::DefaultState, Config, ConfigBuilder, ConfigError, Environment, File, Source,
};
use serde::Serialize;
use serde_derive::Deserialize;
use strum_macros::EnumString;

// You need to bring the ToString trait into scope to use it
use std::string::ToString;
use strum_macros::Display;

use crate::{
    git::get_hooks_path,
    prompt::{
        PROMPT_TO_SUMMARIZE_DIFF, PROMPT_TO_SUMMARIZE_DIFF_SUMMARIES,
        PROMPT_TO_SUMMARIZE_DIFF_TITLE,
    },
};

#[derive(Debug, Clone, Display, Serialize, Default, EnumString)]
pub enum ModelProvider {
    #[default]
    #[strum(serialize = "openai")]
    #[serde(rename = "openai")]
    OpenAI,
}

// implement the trait `From<ModelProvider>` for `ValueKind`
impl From<ModelProvider> for config::ValueKind {
    fn from(model_provider: ModelProvider) -> Self {
        Self::String(model_provider.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ModelProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ModelProviderVisitor;

        impl<'de> serde::de::Visitor<'de> for ModelProviderVisitor {
            type Value = ModelProvider;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an string representing a ModelProvider")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<ModelProvider, E> {
                ModelProvider::from_str(s)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(s), &self))
            }
        }
        deserializer.deserialize_any(ModelProviderVisitor)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct OpenAISettings {
    pub api_key: Option<String>,
    pub model: Option<String>,
}

// implement the trait `From<OpenAISettings>` for `ValueKind`
impl From<OpenAISettings> for config::ValueKind {
    fn from(settings: OpenAISettings) -> Self {
        let mut properties = HashMap::new();
        properties.insert("api_key".to_string(), config::Value::from(settings.api_key));
        properties.insert("model".to_string(), config::Value::from(settings.model));
        Self::Table(properties)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PromptSettings {
    pub file_diff: Option<String>,
    pub commit_summary: Option<String>,
    pub commit_title: Option<String>,
}

// implement the trait `From<OpenAISettings>` for `ValueKind`
impl From<PromptSettings> for config::ValueKind {
    fn from(settings: PromptSettings) -> Self {
        let mut properties = HashMap::new();
        properties.insert(
            "file_diff".to_string(),
            config::Value::from(settings.file_diff),
        );
        properties.insert(
            "commit_summary".to_string(),
            config::Value::from(settings.commit_summary),
        );
        properties.insert(
            "commit_title".to_string(),
            config::Value::from(settings.commit_title),
        );
        Self::Table(properties)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub model_provider: Option<ModelProvider>,
    pub openai: Option<OpenAISettings>,
    pub prompt: Option<PromptSettings>,
    pub allow_amend: Option<bool>,
}

impl Settings {
    pub fn from_clear(key: &str) -> Result<Self, ConfigError> {
        let mut settings = Self::get_config_builder()?;
        settings = settings.set_override(key, None::<Option<String>>)?;
        settings.build()?.try_deserialize()
    }

    pub fn from_set_override(key: &str, value: &str) -> Result<Self, ConfigError> {
        let mut settings = Self::get_config_builder()?;
        settings = settings.set_override(key, value)?;
        settings.build()?.try_deserialize()
    }

    pub fn new() -> Result<Self, ConfigError> {
        let settings = Self::get_config_builder()?;
        settings.build()?.try_deserialize()
    }

    fn get_config_builder() -> Result<ConfigBuilder<DefaultState>, ConfigError> {
        let mut settings = Config::builder()
            .set_default("allow_amend", false)?
            .set_default("model_provider", ModelProvider::OpenAI)?
            .set_default(
                "openai",
                Some(OpenAISettings {
                    api_key: None,
                    model: Some("text-davinci-003".to_string()),
                }),
            )?
            .set_default(
                "prompt",
                Some(PromptSettings {
                    file_diff: Some(PROMPT_TO_SUMMARIZE_DIFF.to_string()),
                    commit_summary: Some(PROMPT_TO_SUMMARIZE_DIFF_SUMMARIES.to_string()),
                    commit_title: Some(PROMPT_TO_SUMMARIZE_DIFF_TITLE.to_string()),
                }),
            )?;

        if let Some(home_dir) = dirs::home_dir() {
            debug!("Using home dir at {}", home_dir.display());

            let config_dir = home_dir.join(".config").join(APP_NAME);
            if config_dir.is_dir() {
                debug!("Found config dir at {}", config_dir.display());

                let config_path = config_dir.join("config.toml");
                if config_path.is_file() {
                    debug!("Applying config file at {}", config_path.display());
                    settings = settings.add_source(File::from(config_path));
                } else {
                    debug!("Config file at {} is not a file", config_path.display());
                }
            } else {
                debug!("Config dir at {} is not a dir", config_dir.display());
            }
        }

        // Add repo-local config
        // Find git repo
        if let Ok(hooks_path) = get_hooks_path() {
            let config_path = hooks_path.join("../gptcommit.toml");
            settings = settings.add_source(File::from(config_path).required(false));
        }

        // Add in settings from the environment (with a prefix of GPTCOMMIT)
        // Eg.. `GPTCOMMIT__DEBUG=1 ./target/app` would set the `debug` key

        let app_env = Environment::with_prefix(APP_NAME).separator("__");
        debug!(
            "Applying config from  GPTCOMMIT__*: {:?}",
            app_env.collect()
        );
        settings = settings.add_source(app_env);

        // add custom override
        if let Ok(openai_api_key) = std::env::var("OPENAI_API_KEY") {
            if !openai_api_key.is_empty() {
                debug!("Applying OPENAI_API_KEY envvar: {}", openai_api_key);
                settings = settings.set_override("openai.api_key", Some(openai_api_key))?;
            }
        }

        Ok(settings)
    }
}

pub fn get_local_config_path() -> Option<PathBuf> {
    if let Ok(config_dir) = get_hooks_path() {
        if !config_dir.is_dir() {
            fs::create_dir_all(&config_dir).ok()?;
        }
        let config_path = config_dir
            .parent()
            .unwrap_or_else(|| &config_dir)
            .join("gptcommit.toml");
        if !config_path.exists() {
            fs::write(&config_path, "").ok()?;
        }
        return Some(config_path);
    }

    None
}

pub fn get_user_config_path() -> Option<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        let config_dir = home_dir.join(".config").join(APP_NAME);
        if !config_dir.is_dir() {
            fs::create_dir_all(&config_dir).ok()?;
        }
        let config_path = config_dir.join("config.toml");
        if !config_path.exists() {
            fs::write(&config_path, "").ok()?;
        }
        return Some(config_path);
    }
    None
}
const APP_NAME: &str = "gptcommit";
