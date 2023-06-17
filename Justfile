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

e2e: install
    sh -eux -c 'for i in ./tests/e2e/test_*.sh ; do sh -x "$i" ; done'

test *args: e2e
    cargo test
alias t := test

bench: install
    sh ./tests/bench/run_bench.sh

lint:
    cargo fmt --all -- --check
    cargo clippy --all-features --all-targets -- -D warnings --allow deprecated

fix:
    cargo fix --allow-dirty --allow-staged
    cargo clippy --all-features --all-targets --fix --allow-dirty --allow-staged -- -D warnings --allow deprecated
    cargo fmt --all
alias f := fix


# Bump version. level=major,minor,patch
version level:
    git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.
    cargo set-version --bump {{level}}
    cargo update # This bumps Cargo.lock
    VERSION=$(toml get Cargo.toml package.version) && \
        git commit -am "Bump version {{level}} to $VERSION" && \
        git push origin HEAD
    git push

release-patch: lint build test
    just version patch

release-minor: lint build test
    just version minor

release-major: lint build test
    just version major
