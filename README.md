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

To use `gptcommit`, simply run `git commit` as you normally would. The hook will automatically generate a commit message for you using a large language model like GPT. If you're not satisfied with the generated message, you can always edit it before committing.

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

### Set a custom OpenAI API base URL

Persist your OpenAI key

```sh
gptcommit config set openai.api_base https://...
```

or set it just for you local repo:

```sh
gptcommit config set --local openai.api_base https://...
```

You can also config this setting via the `GPTCOMMIT__OPENAI__API_BASE` or .

To maintain compatibility with other OpenAI clients, we support the `OPENAI_API_BASE` environment variables. This will take the highest precedence.

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

### Proxy configuration support

Configure an OpenAI proxy using

```sh
gptcommit config set openai.proxy "my_http_proxy...."
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

### Installing in GitHub codespaces

You'll need to install Rust and the cargo toolchain first. Remember to configure your API key.

```sh
curl https://sh.rustup.rs -sSf | sh
bash
cargo install --locked gptcommit

# insert your openai api key https://platform.openai.com/account/api-keys
gptcommit config set openai.api_key # sk-...
```

## Derived Works

All of these awesome projects are built using `gptcommit`.

- A VSCode extension you can
    [install here](https://marketplace.visualstudio.com/items?itemName=pwwang.gptcommit) | [GitHub](https://github.com/pwwang/vscode-gptcommit)

## Encountered any bugs?

If you encounter any bugs or have any suggestions for improvements, please open an issue on the repository.

## License

This project is licensed under the [MIT License](./LICENSE).

---

## Detailed Help Usage

```
$ gptcommit -h
Usage: gptcommit [OPTIONS] <COMMAND>

Commands:
  install             Install the git hook
  uninstall           Uninstall the git hook
  config              Read and modify settings
  prepare-commit-msg  Run on the prepare-commit-msg hook
  help                Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Enable verbose logging
  -h, --help     Print help
  -V, --version  Print version
```

```
$ gptcommit install -h
Install the git hook

Usage: gptcommit install [OPTIONS]

Options:
  -v, --verbose  Enable verbose logging
  -h, --help     Print help
  -V, --version  Print version
```

```
$ gptcommit uninstall -h
Uninstall the git hook

Usage: gptcommit uninstall [OPTIONS]

Options:
  -v, --verbose  Enable verbose logging
  -h, --help     Print help
  -V, --version  Print version
```

```
$ gptcommit config -h
Read and modify settings

Usage: gptcommit config [OPTIONS] <COMMAND>

Commands:
  keys    List all config keys
  list    List all config values
  get     Read a config value
  set     Set a config value
  delete  Clear a config value
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Enable verbose logging
  -h, --help     Print help
  -V, --version  Print version
```

```
$ gptcommit config keys
allow_amend
file_ignore
model_provider
openai.api_base
openai.api_key
openai.model
openai.proxy
openai.retries
output.conventional_commit
optput.conventional_commit_prefix_format
output.lang
output.show_per_file_summary
prompt.commit_summary
prompt.commit_title
prompt.conventional_commit_prefix
prompt.file_diff
prompt.translation
```
