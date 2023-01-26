use colored::Colorize;

pub(crate) fn print_help_openai_api_key() {
    println!(
                "{}",
            r#"OpenAI API key not found in config or environment.

Configure the OpenAI API key with the command:

    export GPTCOMMIT__OPENAI__API_KEY='sk-...'

Or add the following to your ~/.config/gptcommit/config.toml file:
```
model_provider = "openai"

[openai]
api_key = "sk-..."
```

Note: OPENAI_API_KEY will be deprecated in a future release. Please use GPTCOMMIT__OPENAI__API_KEY instead, or a config file.
"#
            .bold()
            .yellow(),
        );
}
