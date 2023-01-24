use std::{collections::HashMap, str::FromStr};

use config::{Config, ConfigError, Environment, File, Source};
use serde_derive::Deserialize;
use strum_macros::EnumString;

// You need to bring the ToString trait into scope to use it
use std::string::ToString;
use strum_macros::Display;

#[derive(Debug, Clone, Display, Default, EnumString)]
pub enum ModelProvider {
    #[default]
    #[strum(serialize = "openai")]
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

#[derive(Debug, Default, Deserialize, Clone)]
pub struct OpenAISettings {
    pub api_key: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Settings {
    pub model_provider: Option<ModelProvider>,
    pub openai: Option<OpenAISettings>,
}

// implement the trait `From<OpenAISettings>` for `ValueKind`
impl From<OpenAISettings> for config::ValueKind {
    fn from(settings: OpenAISettings) -> Self {
        let mut properties = HashMap::new();
        properties.insert("api_key".to_string(), config::Value::from(settings.api_key));
        Self::Table(properties)
    }
}

const APP_NAME: &str = "gptcommit";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::builder()
            .set_default("model_provider", ModelProvider::OpenAI)?
            .set_default("open_ai", OpenAISettings { api_key: None })?;

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

        // Freeze the entire configuration
        settings.build()?.try_deserialize()
    }
}
