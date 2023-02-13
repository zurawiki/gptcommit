#!/bin/sh
set -eu

gptcommit config list
# assert is valid TOML

gptcommit config get openai.model
# assert default = text-davinci-003
gptcommit config set --local openai.model foo
gptcommit config set openai.model bar
gptcommit config get openai.model
# assert is foo

gptcommit config delete openai.model
gptcommit config get openai.model
# assert still is foo
gptcommit config delete --local openai.model
gptcommit config get openai.model
# assert is default
