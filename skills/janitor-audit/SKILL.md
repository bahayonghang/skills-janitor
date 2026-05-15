---
name: janitor-audit
description: "Show all your installed skills. Use when the user asks for a skill inventory, skill list, or wants to audit what's installed."
metadata:
  version: 2.0.2
---

## CLI requirement

Before doing anything else, check whether the Rust CLI is installed:

```bash
skills-janitor --version
```

If it is not installed, stop and tell the user to install it with Cargo:

```bash
cargo install skills-janitor --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
```

If the user does not have Rust/Cargo, tell them to download the latest GitHub Release binary for their platform instead. Do not fall back to Python, Bash, or curl scripts unless the user explicitly asks for legacy mode.
# Skill Audit

Run a full inventory scan of all Claude Code skills across every scope.

If the user asks for an HTML report, visual report, pretty report, dashboard, or something they can open in a browser, prefer the integrated dashboard:

```bash
skills-janitor dashboard --open
```

Only add `--weeks N` when the user explicitly asks for a custom analysis period.

If the dashboard command fails with `unexpected argument '--weeks'`, retry once without `--weeks` and report that the installed CLI is stale; users should update to `skills-janitor` 2.0.1 or newer.

If it fails with `Dashboard template not found`, tell the user to install or update to the embedded-template CLI build (`skills-janitor` 2.0.1 or newer). Raw `scan --json` is a fallback for inventory data, not the preferred dashboard path.

Use plain `scan --json` only when they specifically want raw inventory data.

## How to Run

```bash
skills-janitor scan --json
```

Legacy fallback only when explicitly requested:

```bash
bash <scripts_dir>/scan.sh
```

## What It Scans

- **User scope**: `~/.claude/skills/`
- **Project scope**: `./.claude/skills/` (current project)
- **Plugin skills**: `~/.claude/plugins/` and marketplace sources
- **Source links**: `~/.claude/sources/`
- **Account-level**: `~/.claude-account-personal/plugins/`, `~/.claude-account-company/plugins/`

## Output

JSON inventory with per-skill details:
- Folder name, scope, full path
- Symlink status (valid/broken/target)
- Frontmatter fields (name, description, version)
- Body content presence
- Line counts, extra files

## After Scanning

Present findings as a summary table:

```
| Skill              | Scope   | Status   | Issues                    |
|--------------------|---------|----------|---------------------------|
| marketing-copy     | user    | OK       | -                         |
| seo-audit          | user    | WARNING  | Description too short     |
| old-deploy-helper  | user    | CRITICAL | Broken symlink            |
```

## Related Skills

- For duplicate detection: `/janitor-report`
- For auto-fixing issues: `/janitor-fix`
- For a full health report: `/janitor-report`
- For token cost: `/janitor-tokens`
- For a visual dashboard: `skills-janitor dashboard --open`
