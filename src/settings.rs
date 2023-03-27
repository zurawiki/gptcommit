use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};
#[cfg(unix)]
use std::{fs::Permissions, os::unix::prelude::PermissionsExt};

use async_openai::API_BASE;
use config::{
    builder::DefaultState, Config, ConfigBuilder, ConfigError, Environment, File, Source,
};
use serde::Serialize;
use serde_derive::Deserialize;
use strum_macros::EnumString;

// You need to bring the ToString trait into scope to use it
use std::string::ToString;
use strum_macros::{Display, IntoStaticStr};

use crate::{
    git::get_hooks_path,
    prompt::{
        PROMPT_TO_CONVENTIONAL_COMMIT_PREFIX, PROMPT_TO_SUMMARIZE_DIFF,
        PROMPT_TO_SUMMARIZE_DIFF_SUMMARIES, PROMPT_TO_SUMMARIZE_DIFF_TITLE, PROMPT_TO_TRANSLATE,
    },
};

static DEFAULT_OPENAI_MODEL: &str = "gpt-3.5-turbo";

static DEFAULT_FILES_TO_IGNORE: &[&str; 4] = &[
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
];

#[derive(Debug, Clone, Display, Serialize, Default, EnumString)]
pub(crate) enum ModelProvider {
    #[default]
    #[strum(serialize = "openai")]
    #[serde(rename = "openai")]
    OpenAI,
    #[strum(serialize = "tester-foobar")]
    #[serde(rename = "tester-foobar")]
    TesterFoobar,
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

#[derive(Default, Serialize, Deserialize, Clone)]
pub(crate) struct OpenAISettings {
    pub api_base: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub retries: Option<u16>,
    pub proxy: Option<String>,
}

impl std::fmt::Debug for OpenAISettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAISettings")
            .field("api_base", &self.api_base)
            .field(
                "api_key",
                // obfuscate the api key
                &self.api_key.as_ref().map(|_| "********"),
            )
            .field("model", &self.model)
            .field("retries", &self.retries)
            .field("proxy", &self.proxy)
            .finish()
    }
}

// implement the trait `From<OpenAISettings>` for `ValueKind`
impl From<OpenAISettings> for config::ValueKind {
    fn from(settings: OpenAISettings) -> Self {
        let mut properties = HashMap::new();
        properties.insert(
            "api_base".to_string(),
            config::Value::from(settings.api_base),
        );
        properties.insert("api_key".to_string(), config::Value::from(settings.api_key));
        properties.insert("model".to_string(), config::Value::from(settings.model));
        properties.insert("retries".to_string(), config::Value::from(settings.retries));
        properties.insert("proxy".to_string(), config::Value::from(settings.proxy));
        Self::Table(properties)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub(crate) struct PromptSettings {
    pub conventional_commit_prefix: Option<String>,
    pub commit_summary: Option<String>,
    pub commit_title: Option<String>,
    pub file_diff: Option<String>,
    pub translation: Option<String>,
}

// implement the trait `From<PromptSettings>` for `ValueKind`
impl From<PromptSettings> for config::ValueKind {
    fn from(settings: PromptSettings) -> Self {
        let mut properties = HashMap::new();

        properties.insert(
            "conventional_commit_prefix".to_string(),
            config::Value::from(settings.conventional_commit_prefix),
        );
        properties.insert(
            "commit_summary".to_string(),
            config::Value::from(settings.commit_summary),
        );
        properties.insert(
            "commit_title".to_string(),
            config::Value::from(settings.commit_title),
        );
        properties.insert(
            "file_diff".to_string(),
            config::Value::from(settings.file_diff),
        );
        properties.insert(
            "translation".to_string(),
            config::Value::from(settings.translation),
        );
        Self::Table(properties)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Language {
    #[default]
    #[strum(serialize = "en")]
    #[strum(to_string = "English")]
    En,
    #[strum(serialize = "zh-cn")]
    #[strum(to_string = "Simplified Chinese")]
    ZhCn,
    #[strum(serialize = "zh-tw")]
    #[strum(to_string = "Traditional Chinese")]
    ZhTw,
    #[strum(serialize = "ja")]
    #[strum(to_string = "Japanese")]
    Ja,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct OutputSettings {
    /// Whether to add a conventional commit tag to the commit message
    pub conventional_commit: Option<bool>,
    /// Output language of the commit message
    pub lang: Option<String>,
    /// Whether to show the summary of each file in the commit
    pub show_per_file_summary: Option<bool>,
}

// implement the trait `From<OutputSettings>` for `ValueKind`
impl From<OutputSettings> for config::ValueKind {
    fn from(settings: OutputSettings) -> Self {
        let mut properties = HashMap::new();
        properties.insert(
            "conventional_commit".to_string(),
            config::Value::from(settings.conventional_commit),
        );
        properties.insert("lang".to_string(), config::Value::from(settings.lang));
        properties.insert(
            "show_per_file_summary".to_string(),
            config::Value::from(settings.show_per_file_summary),
        );
        Self::Table(properties)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub(crate) struct Settings {
    pub model_provider: Option<ModelProvider>,
    pub openai: Option<OpenAISettings>,
    pub prompt: Option<PromptSettings>,
    pub output: Option<OutputSettings>,
    /// Whether to run githook when amending the commit
    pub allow_amend: Option<bool>,
    /// Files to ignore, format similar to gitignore
    pub file_ignore: Option<Vec<String>>,
}

impl Settings {
    pub fn from_clear(key: &str) -> Result<Self, ConfigError> {
        let mut settings = Self::get_config_builder()?;
        settings = settings.set_override(key, None::<Option<String>>)?;
        settings.build()?.try_deserialize()
    }

    pub fn from_set_override(key: &str, value: &str) -> Result<Self, ConfigError> {
        if key == "output.lang" && Language::from_str(value).is_err() {
            return Err(ConfigError::Message(format!("Invalid language: {value}.",)));
        }
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
            .set_default(
                "file_ignore",
                DEFAULT_FILES_TO_IGNORE
                    .to_vec()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            )?
            .set_default("model_provider", ModelProvider::OpenAI)?
            .set_default(
                "openai",
                Some(OpenAISettings {
                    api_base: Some(API_BASE.to_string()),
                    api_key: None,
                    model: Some(DEFAULT_OPENAI_MODEL.to_string()),
                    retries: Some(2),
                    proxy: Some("".to_string()),
                }),
            )?
            .set_default(
                "prompt",
                Some(PromptSettings {
                    conventional_commit_prefix: Some(
                        PROMPT_TO_CONVENTIONAL_COMMIT_PREFIX.to_string(),
                    ),
                    file_diff: Some(PROMPT_TO_SUMMARIZE_DIFF.to_string()),
                    commit_summary: Some(PROMPT_TO_SUMMARIZE_DIFF_SUMMARIES.to_string()),
                    commit_title: Some(PROMPT_TO_SUMMARIZE_DIFF_TITLE.to_string()),
                    translation: Some(PROMPT_TO_TRANSLATE.to_string()),
                }),
            )?
            .set_default(
                "output",
                Some(OutputSettings {
                    conventional_commit: Some(true),
                    lang: Some("en".to_string()),
                    show_per_file_summary: Some(false),
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
        if let Ok(openai_api_base) = std::env::var("OPENAI_API_BASE") {
            if !openai_api_base.is_empty() {
                debug!("Applying OPENAI_API_BASE envvar: {}", openai_api_base);
                settings = settings.set_override("openai.api_base", Some(openai_api_base))?;
            }
        }
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
            .unwrap_or(&config_dir)
            .join("gptcommit.toml");
        if !config_path.exists() {
            fs::write(&config_path, "").ok()?;
            #[cfg(unix)]
            fs::set_permissions(&config_path, Permissions::from_mode(0o600)).unwrap();
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
            #[cfg(unix)]
            fs::set_permissions(&config_path, Permissions::from_mode(0o600)).unwrap();
        }
        return Some(config_path);
    }
    None
}
const APP_NAME: &str = "gptcommit";
