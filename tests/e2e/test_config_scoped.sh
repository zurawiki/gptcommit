#!/bin/sh
set -eu

config_test_dir=$(mktemp -d)
trap 'rm -rf "$config_test_dir"' EXIT
cd "$config_test_dir"
git init

printf '# local preferences\n' > .git/gptcommit.toml
GPTCOMMIT__OPENAI__MODEL=inherited-model \
GPTCOMMIT__OPENAI__API_KEY=test-only-not-a-secret \
    gptcommit config set --local output.lang en
grep -q '^# local preferences$' .git/gptcommit.toml
if grep -q 'inherited-model\|test-only-not-a-secret\|api_key\|prompt' .git/gptcommit.toml; then
    exit 1
fi

gptcommit config set --local allow_amend true
test "$(gptcommit config get allow_amend)" = true
gptcommit config set --local openai.retries 3
test "$(gptcommit config get openai.retries)" = 3
gptcommit config set --local openai.model local-model
gptcommit config delete --local openai.model
test "$(GPTCOMMIT__OPENAI__MODEL=inherited-model gptcommit config get openai.model)" = inherited-model
if grep -q 'model\|inherited-model' .git/gptcommit.toml; then
    exit 1
fi
test "$(gptcommit config get openai.retries)" = 3
