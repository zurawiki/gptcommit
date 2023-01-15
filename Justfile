set positional-arguments
set dotenv-load := true

help:
    @just --list --unsorted

clean:
    cargo clean

build:
    cargo build
alias b := build

run *args:
    cargo run -- "$@"
alias r := run

release:
    cargo build --release

install:
    cargo install --path .

# Tools for development
bootstrap:
    cargo install cargo-edit
    cargo install --git https://github.com/kurtbuilds/toml-cli

test *args:
    cargo test
alias t := test

check:
    cargo check
alias c := check

fix:
    cargo clippy --fix

bench:
    cargo criterion --features bench

# Bump version. level=major,minor,patch
version level:
    git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.
    cargo set-version --bump {{level}}
    cargo update # This bumps Cargo.lock
    VERSION=$(toml get Cargo.toml package.version) && \
        git commit -am "Bump version {{level}} to $VERSION" && \
        git tag v$VERSION && \
        git push origin v$VERSION
    git push

publish:
    cargo publish

patch: test
    just version patch
    just publish
