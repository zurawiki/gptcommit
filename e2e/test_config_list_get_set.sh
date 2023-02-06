#!/bin/sh
set -eu

gptcommit config list
# assert is valid TOML

gptcommit config get openai.model
# assert default = text-davinci-003
gptcommit config set openai.model foo
gptcommit config get openai.model
# assert is foo

gptcommit config delete openai.model
gptcommit config get openai.model
# back to default
