# Cross-platform task runner for Skills Janitor.

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-targets --all-features

build:
    cargo build --workspace

build-release:
    cargo build --release

release-build: build-release

ci: fmt-check clippy test build-release

install-local:
    cargo install --path . --locked --force

run *args:
    cargo run --bin skills-janitor -- {{args}}
