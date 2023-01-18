# `gptcommmit`

[![Github Contributors](https://img.shields.io/github/contributors/zurawiki/gptcommit.svg)](https://github.com/zurawiki/gptcommit/graphs/contributors)
[![Github Stars](https://img.shields.io/github/stars/zurawiki/gptcommit.svg)](https://github.com/zurawiki/gptcommit/stargazers)
[![Github Build Status](https://img.shields.io/github/actions/workflow/status/zurawiki/gptcommit/test.yaml)](https://github.com/zurawiki/gptcommit/actions)


[![crates.io status](https://img.shields.io/crates/v/gptcommit.svg)](https://crates.io/crates/gptcommit)
[![crates.io downloads](https://img.shields.io/crates/d/gptcommit.svg)](https://crates.io/crates/gptcommit)
[![Rust dependency status](https://deps.rs/repo/github/zurawiki/gptcommit/status.svg)](https://deps.rs/repo/github/zurawiki/gptcommit)

A git prepare-commit-msg hook for authoring commit messages with GPT-3. With this tool, you can easily generate clear, comprehensive and descriptive commit messages letting you focus on writing code.

See [announcement blog post](https://zura.wiki/post/never-write-a-commit-message-again-with-the-help-of-gpt-3/).

## Demo

[![asciicast](https://asciinema.org/a/552380.svg)](https://asciinema.org/a/552380)

## Installation
1. Install this tool locally with cargo (recommended) or homebrew.
```
cargo install gptcommit
```

2. In your `git` repository, run the following command to install `gptcommit` as a git prepare-commit-msg hook. You will need to provide an OpenAI API key to complete the installation.

```
gptcommit install
```

## Usage
To use `gptcommit`, simply run `git commit` as you normally would. The hook will automatically generate a commit message for you using GPT-3. If you're not satisfied with the generated message, you can always edit it before committing.

Note: By default, `gptcommit` uses the GPT-3 model. Please ensure you have sufficient credits in your OpenAI account to use it.

**n.b.** If your OpenAI API key has changed, simply re-run `gptcommit install`.

## Encountered any bugs?

If you encounter any bugs or have any suggestions for improvements, please open an issue on the repository.

## License

This project is licensed under the [MIT License](./LICENSE).
