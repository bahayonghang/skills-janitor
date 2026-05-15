# Cross-platform task runner for Skills Janitor.

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

sync-rust-metadata:
    cargo generate-lockfile
    cargo test --test package_identity -- package_metadata_has_single_root_package --exact

sync-skill-version:
    cargo test --test version_sync -- sync_janitor_audit_skill_version --ignored --exact --nocapture

sync: sync-rust-metadata sync-skill-version

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-targets --all-features

build:
    cargo build

build-release:
    cargo build --release

release-build: build-release

ci: sync fmt-check clippy test build-release

install *args:
    cargo install --path . --locked --force
    cargo run --bin skills-janitor -- install-skills {{args}}

install-local *args: (install args)

run *args:
    cargo run --bin skills-janitor -- {{args}}
