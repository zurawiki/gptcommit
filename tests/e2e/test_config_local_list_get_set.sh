#!/bin/sh
set -eu

export TEMPDIR=$(mktemp -d)
(
    cd "${TEMPDIR}"
    git init

    gptcommit config list
    # assert is valid TOML

    gptcommit config get openai.model
    # assert default = gpt-4.1-nano
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
)
rm -rf "${TEMPDIR}"


export TEMPDIR=$(mktemp -d)
(
    cd "${TEMPDIR}"

    gptcommit config list
    # assert is valid TOML

    gptcommit config get openai.model
    # assert default = gpt-4.1-nano
    set +e
    gptcommit config set --local openai.model foo
    # TODO assert output
    test $? -ne 0 || exit $?
    set -e
    gptcommit config set openai.model bar
    gptcommit config get openai.model
    # assert is foo

    gptcommit config delete openai.model
    gptcommit config get openai.model
    # assert still is foo
    set +e
    gptcommit config delete --local openai.model
    # TODO assert output
    test $? -ne 0 || exit $?
    set -e
    gptcommit config get openai.model
    # assert is default
)
rm -rf "${TEMPDIR}"
