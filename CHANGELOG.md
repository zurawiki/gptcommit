# Changelog

## [0.6.0]

### Upgrade notes

- Building from source now requires Rust 1.88 or newer.
- `file_ignore` now uses Git-style patterns instead of substring matching. Use
  `*.lock` for a filename suffix, `/path` for a repository-root path, and ordered
  `!` rules to include exceptions. Review custom rules when upgrading; for
  example, a substring rule `lock` should become `*.lock` to exclude lockfiles.
  These rules are separate from the repository's `.gitignore`.
- The default OpenAI model is now `gpt-6-luna`, with low reasoning effort and a
  2,048-token limit shared by reasoning and output for each request. Explicit model
  overrides are preserved and do not receive these model-specific options.
  Choose another model with `gptcommit config set openai.model <model>` if needed.

### Improvements

- Configuration reads display booleans, numbers, and arrays correctly. Setting or
  deleting a key edits only that key in the selected file, preserves comments,
  and avoids copying environment or unrelated configuration overrides into it.
- Ignore rules support globs, directories, anchored paths, and negation. Verbose
  output identifies excluded files and the rules that matched them.
- Empty or fully ignored changes leave the commit message untouched and make no
  API calls. Routine progress output is quieter; use `--verbose` for details.
- File summarization runs at most four requests concurrently, preserves stable
  output order, and reports failures instead of silently omitting summaries.
- Default prompts request concise, factual titles and bullets without invented
  motivation or test claims. Bun and uv lockfiles are excluded by default.
- OpenAI client and dependency updates improve compatibility and retry handling;
  errors are reported without panics, with tokenization fallback for unknown models.

### Release maintenance

- Updated GitHub Actions are pinned to commit hashes.
- Homebrew and Winget updates wait until all release binaries are uploaded,
  removing the race with archive downloads.

[0.6.0]: https://github.com/zurawiki/gptcommit/compare/v0.5.17...v0.6.0
