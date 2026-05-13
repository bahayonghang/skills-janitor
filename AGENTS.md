# AGENTS.md

## Repo shape

- Single Rust crate, not a Cargo workspace. `Cargo.toml` defines one binary: `skills-janitor` at `src/main.rs`.
- Rust minimum is 1.82. `Cargo.lock` is tracked and the CI sync step starts with `cargo generate-lockfile`.
- `src/main.rs` only delegates to `cli::run()`. The real command surface lives in `src/cli.rs`.
- README slash commands are wrappers. Implement behavior in the Rust subcommands: `scan`, `report`, `fix`, `usage`, `tokens`, `search`, `compare`, `precheck`, `dashboard`.
- `tests/package_identity.rs` intentionally enforces exactly one root package named `skills-janitor`; do not reintroduce a split `cli` package without changing that guardrail.

## Commands

- Full local check: `just ci`.
- `just ci` order is: `sync-rust-metadata` -> `fmt-check` -> `clippy` -> `test` -> `build-release`.
- CI-equivalent without `just`:
  1. `cargo generate-lockfile`
  2. `cargo test --test package_identity -- package_metadata_has_single_root_package --exact`
  3. `cargo fmt --all -- --check`
  4. `cargo clippy --all-targets --all-features -- -D warnings`
  5. `cargo test --all-targets --all-features`
  6. `cargo build --release`
- Run the CLI from source with `just run <subcommand> ...` or `cargo run --bin skills-janitor -- <subcommand> ...`.
- Focused tests: `cargo test <test_name>`, `cargo test --test dashboard_cli`, or the exact package identity command above.
- Local install is `just install-local`, which runs `cargo install --path . --locked --force`.

## Behavior gotchas

- `fix` is dry-run unless `--apply` is present; `--dry-run` forces preview mode even with other flags.
- `fix --prune` adds broken symlinks and empty skill directories to the plan. Plugin and marketplace skill paths are deliberately skipped.
- Skill roots are hardcoded in `src/paths.rs`: `~/.claude/skills`, `./.claude/skills`, `~/.agents/skills`, and `./.agents/skills`.
- Usage history is read from Claude history locations under `~/.claude` and `~/.claude-account-personal`; Codex history is not currently scanned there.
- Inventory and fix logic skip a skill folder named `skills-janitor` to avoid modifying the tool's own installed skill.
- `dashboard` embeds `assets/janitor-dashboard.html`. By default it writes `janitor-dashboard.html` in the current working directory, so use `--output` for manual checks and tests.
- Dashboard snapshots are stored inside the generated HTML and capped at the latest 20 snapshots.

## Docs and release

- `docs/` is a separate VitePress site. Run `npm install`, `npm run dev`, or `npm run build` from `docs/`; root CI does not build it.
- Keep README, `docs/cli.md`, and `docs/skills.md` aligned when changing CLI flags or subcommands.
- Release workflow runs on `v*` tags and archives the built binary with `README.md` and `LICENSE` for Windows, Linux, and macOS targets.
