#!/bin/sh
set -eu

(
    rm -rf test_dir_foo
    mkdir test_dir_foo
    cd test_dir_foo
    git init
    gptcommit install

    # assert that git hook is installed
    gptcommit install
    # assert still works
)
rm -rf test_dir_foo ;

#############################

(
    rm -rf test_dir_foo2
    mkdir test_dir_foo2
    cd test_dir_foo2
    git init
    mkdir a
    cd a
    gptcommit install
)
rm -rf test_dir_foo2

#############################

(
    rm -rf test_dir_foo3
    mkdir test_dir_foo3
    cd test_dir_foo3
    # no git init
    gptcommit install ;
)
rm -rf test_dir_foo3
