#!/bin/sh

GPTCOMMIT__MODEL_PROVIDER="tester-foobar" \
gptcommit prepare-commit-msg \
    --git-diff-content "${1}" \
    --commit-msg-file "${2}" \
    --commit-source ""
