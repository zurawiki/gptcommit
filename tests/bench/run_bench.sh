#!/bin/sh
set -eu

ROOT="$(pwd)"
DIFF_CONTENT_PATH="$(pwd)/tests/data/example_1.diff"

export TEMPDIR=$(mktemp -d)
(
    cd "${TEMPDIR}"
    git init

    export TEMPFILE=$(mktemp)
    echo "foo" > $TEMPFILE

    hyperfine \
        --warmup 1 \
        --max-runs 10 \
        "$ROOT/tests/bench/bench_githook_test.sh"\ "${DIFF_CONTENT_PATH}"\ "${TEMPFILE}" \
        "$ROOT/tests/bench/bench_githook_openai.sh"\ "${DIFF_CONTENT_PATH}"\ "${TEMPFILE}" \
)
rm -rf "${TEMPDIR}"
