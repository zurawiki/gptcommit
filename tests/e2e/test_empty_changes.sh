#!/bin/sh
set -eu

defaults_test_dir=$(mktemp -d)
trap 'rm -rf "$defaults_test_dir"' EXIT
cd "$defaults_test_dir"
git init
printf 'Existing message\n' > message.txt
printf 'file_ignore = ["*.lock"]\n' > .git/gptcommit.toml

for state in empty ignored; do
    if [ "$state" = ignored ]; then
        printf 'generated\n' > generated.lock
        git add generated.lock
    fi
    output=$(OPENAI_API_KEY='' GPTCOMMIT__OPENAI__API_KEY='' \
        gptcommit prepare-commit-msg --commit-msg-file message.txt --commit-source '')
    test -z "$output"
    test "$(cat message.txt)" = 'Existing message'
done
