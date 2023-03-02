#!/bin/sh

gptcommit prepare-commit-msg \
    --git-diff-content "${1}" \
    --commit-msg-file "${2}" \
    --commit-source ""
