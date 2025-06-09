#!/bin/sh
set -eu

gptcommit config list
# assert is valid TOML
gptcommit config keys

gptcommit config get openai.model
# assert default = gpt-4.1-nano
gptcommit config set openai.model foo
gptcommit config get openai.model
# assert is foo

gptcommit config delete openai.model
gptcommit config get openai.model
# back to default
