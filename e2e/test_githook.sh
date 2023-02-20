#!/bin/sh
set -eu

DIFF_CONTENT_PATH="$(pwd)/tests/data/example_1.diff"

export TEMPDIR=$(mktemp -d)
(
    cd "${TEMPDIR}"
    git init

    export TEMPFILE=$(mktemp)
    echo "foo" > $TEMPFILE

    GPTCOMMIT__OPENAI__MODEL="text-ada-001" \
    gptcommit prepare-commit-msg \
      --git-diff-content "${DIFF_CONTENT_PATH}" \
      --commit-msg-file "${TEMPFILE}" \
      --commit-source ""

    cat $TEMPFILE
)
rm -rf "${TEMPDIR}"
