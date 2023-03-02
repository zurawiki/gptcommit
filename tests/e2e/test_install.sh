#!/bin/sh
set -eu

export TEMPDIR=$(mktemp -d)
(
    cd "${TEMPDIR}"
    git init
    gptcommit install

    # assert that git hook is installed
    gptcommit install
    # assert still works
)
rm -rf "${TEMPDIR}"

#############################

export TEMPDIR=$(mktemp -d)
(
    cd "${TEMPDIR}"
    git init
    mkdir a
    cd a
    gptcommit install
)
rm -rf "${TEMPDIR}"

#############################

export TEMPDIR=$(mktemp -d)
(
    cd "${TEMPDIR}"
    # no git init
    set +e
    gptcommit install ;
    # TODO assert output
    test $? -ne 0 || exit $?
    set -e
)
rm -rf "${TEMPDIR}"
