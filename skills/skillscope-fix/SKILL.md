---
name: skillscope-fix
description: "Automatically fix skill problems (safe preview first). Also use with --prune to find and remove broken symlinks, empty directories, and orphaned skills."
metadata:
  version: 2.0.1
---

## CLI requirement

Before doing anything else, check whether the Rust CLI is installed:

```bash
skillscope --version
```

If it is not installed, stop and tell the user to install it with Cargo:

```bash
cargo install skillscope --git https://github.com/bahayonghang/skillscope --bin skillscope --locked --force
```

If the user does not have Rust/Cargo, tell them to download the latest GitHub Release binary for their platform instead. Do not fall back to Python, Bash, or curl scripts unless the user explicitly asks for legacy mode.
# Auto-Fix

Automatically fix common skill issues. Dry-run by default - shows what would change without modifying files.

## How to Run

```bash
skillscope fix                 # preview fixes
skillscope fix --apply          # apply fixes
skillscope fix --prune          # find broken/orphaned skills
skillscope fix --prune --apply  # remove broken skills
```

Legacy fallback only when explicitly requested:

```bash
bash <scripts_dir>/fix.sh [--apply] [--prune]
```

## What It Fixes

- Adds missing frontmatter delimiters (`---`)
- Fills empty `description` fields with a template
- Adds missing `version` field (defaults to "1.0.0")
- Generates template descriptions using the skill folder name

## Prune Mode (`--prune`)

Finds and removes broken skills:
- **Broken symlinks** - skill folder points to deleted source
- **Empty directories** - skill folder with no SKILL.md
- **Orphaned skills** - user-scope copies of plugin skills

Dry-run by default. Pass `--apply` to actually remove them.

## Safety

- **Dry-run by default** - must pass `--apply` to write changes
- Skips plugin/marketplace skills (changes get overwritten on update)
- Skips broken symlinks (unless `--prune` mode)
- Logs ALL changes with timestamps to `data/changelog.log`
- Always asks for confirmation before removing

## Related Skills

- For finding issues: `/skillscope-report`
- For usage analytics: `/skillscope-usage`
- For token cost: `/skillscope-tokens`
