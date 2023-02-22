# `gptcommit`

[![Github Contributors](https://img.shields.io/github/contributors/zurawiki/gptcommit.svg)](https://github.com/zurawiki/gptcommit/graphs/contributors)
[![Github Stars](https://img.shields.io/github/stars/zurawiki/gptcommit.svg)](https://github.com/zurawiki/gptcommit/stargazers)
[![CI](https://github.com/zurawiki/gptcommit/actions/workflows/ci.yml/badge.svg)](https://github.com/zurawiki/gptcommit/actions/workflows/ci.yml)

[![crates.io status](https://img.shields.io/crates/v/gptcommit.svg)](https://crates.io/crates/gptcommit)
[![crates.io downloads](https://img.shields.io/crates/d/gptcommit.svg)](https://crates.io/crates/gptcommit)
[![Rust dependency status](https://deps.rs/repo/github/zurawiki/gptcommit/status.svg)](https://deps.rs/repo/github/zurawiki/gptcommit)

A git prepare-commit-msg hook for authoring commit messages with GPT-3. With this tool, you can easily generate clear, comprehensive and descriptive commit messages letting you focus on writing code.

See [announcement blog post](https://zura.wiki/post/never-write-a-commit-message-again-with-the-help-of-gpt-3/).

## Demo

[![asciicast](https://asciinema.org/a/552380.svg)](https://asciinema.org/a/552380)

## Installation

1. Install this tool locally with `cargo` (recommended).

```sh
cargo install --locked gptcommit
```

or on macOS, use homebrew

```sh
brew install zurawiki/brews/gptcommit
```

2. In your `git` repository, run the following command to install `gptcommit` as a git prepare-commit-msg hook. You will need to provide an OpenAI API key to complete the installation.

```
gptcommit install
```

## Usage

To use `gptcommit`, simply run `git commit` as you normally would. The hook will automatically generate a commit message for you using GPT-3. If you're not satisfied with the generated message, you can always edit it before committing.

Note: By default, `gptcommit` uses the GPT-3 model. Please ensure you have sufficient credits in your OpenAI account to use it.

## Features

`gptcommit` supports a number of configuration options that are read from `$HOME/.config/gptcommit/config.toml`.
Configs are applied in the following order:

- User settings as read from `$HOME/.config/gptcommit/config.toml`.
- The settings as read from the repo clone at `$GIT_ROOT/.git/gptcommit.toml`.
- Environment variables starting with `GPTCOMMIT__*`.

See all the config options available with `gptcommit config keys`.

### Set your OpenAI API key

Persist your OpenAI key

```sh
gptcommit config set openai.api_key sk-...
```

or set it just for you local repo:

```sh
gptcommit config set --local openai.api_key sk-...
```

You can also config this setting via the `GPTCOMMIT__OPENAI__API_KEY`.

To maintain compatibility with other OpenAI clients, we support the `OPENAI_API_KEY` environment variables. This will take the highest precedence.

### Try out a different OpenAI model

`gptcommit` uses `text-davinci-003` by default. The model can be configured to use other models as below

```sh
gptcommit config set openai.model text-davinci-002
```

You can also config this setting via the `GPTCOMMIT__OPENAI__MODEL`.

For a list of public OpenAI models, checkout the [OpenAI docs](https://beta.openai.com/docs/models/overview). You can also bring in your own fine-tuned model.

### Set summarizing language

`gptcommit` uses English by default. The language can be configured to use other languages as below

```sh
gptcommit config set output.lang zh-cn
```

Now, supported languages are:
|locale code|language|
|-|-|
|`en`|English|
|`zh-cn`|简体中文|
|`zh-tw`|繁體中文|
|`ja`|日本語|

### Allow re-summarizing when amending commits

```sh
gptcommit config set allow-amend true
```

## Common Issues / FAQs

### How can I reduce my OpenAI usage bill?

In the current design, gptcommit issues N+2 prompts, where N is the number of modified files with diffs under the max_token_limit. The other prompts are the title and summary.

OpenAI Completions are billed by "tokens" that are both sent and generated. Pricing per token depends on the model used. The number of tokens generated are generally predictable (as a commit message is usually only so big) but gptcommit could be sending over a lot of tokens in the form of diff data.

Today, I see two low-hanging solutions for reducing cost:

- Switch to a different model using the openai.model configuration option
- Reduce the side of prompts and diff data sent to OpenAI

OpenAI's pricing page can be found at
<https://openai.com/api/pricing/#faq-completions-pricing>

### The githook is not running when I commit

By default, the githook is only run for new commits.
If a template is set or the commit is being amended, the githook will skip by default.

Because the githook detected the user is supplying their own template, we make sure not to overwrite it with GPT. You can remove the commit template by making sure `git config --local commit.template` is blank.

You can allow gptcommit to summarize amended commits with the following configuration above.

## Derived Works

All of these awesome projects are built using  `gptcommit`.

- A VSCode extension you can
  [install here](https://marketplace.visualstudio.com/items?itemName=pwwang.gptcommit) | [GitHub](https://github.com/pwwangvscode-gptcommit)

## Encountered any bugs?

If you encounter any bugs or have any suggestions for improvements, please open an issue on the repository.

## License

This project is licensed under the [MIT License](./LICENSE).
