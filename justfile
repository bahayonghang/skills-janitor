# Cross-platform task runner for Skills Janitor.

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

sync-rust-metadata:
    cargo generate-lockfile
    cargo test --test package_identity -- package_metadata_has_single_root_package --exact

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-targets --all-features

build:
    cargo build

build-release:
    cargo build --release

release-build: build-release

ci: sync-rust-metadata fmt-check clippy test build-release

install:
    cargo install --path . --locked --force

install-local: install

run *args:
    cargo run --bin skills-janitor -- {{args}}
