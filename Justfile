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

test *args:
    cargo test
alias t := test

lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

fix:
    cargo fix --allow-dirty --allow-staged
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings
    cargo fmt --all

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
