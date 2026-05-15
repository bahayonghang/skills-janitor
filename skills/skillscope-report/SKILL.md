---
name: skillscope-report
description: "Full health check of all your skills in one report. Use when the user wants to check for errors, find duplicates, detect broken skills, or get a complete overview of skill health."
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
# Health Report

Generate a comprehensive health report combining inventory, quality checks, duplicate detection, and broken skill findings.

## How to Run

Run the unified health report:

```bash
skillscope report
```

For machine-readable output:

```bash
skillscope report --json
```

Legacy fallback only when explicitly requested:

```bash
bash <scripts_dir>/scan.sh
bash <scripts_dir>/lint.sh
bash <scripts_dir>/detect_dupes.sh
```

## What It Covers

### Inventory (`skillscope scan`)
- All skills across user, project, plugin, and account scopes
- Symlink status, frontmatter fields, line counts

### Quality Checks (`skillscope report`)
- **Critical**: Broken symlinks, missing SKILL.md, missing frontmatter
- **Warning**: Missing/empty name or description, description too short/long, missing version
- **Info**: No body content, no Gotchas section, large files

### Duplicate Detection (`skillscope report`)
- Keyword overlap analysis using Jaccard similarity
- Flags pairs with >30% overlap
- Shows shared keywords and scopes

### Broken & Orphaned Skills
- Broken symlinks (target deleted)
- Empty directories (no SKILL.md)
- Orphaned user-scope copies of plugin skills

## Report Format

Present a unified report with severity levels:

```
| Skill              | Scope   | Status      | Issues                          |
|--------------------|---------|-------------|---------------------------------|
| marketing-copy     | user    | OK          | -                               |
| seo-audit          | user    | WARNING     | Description too short (28 chars) |
| old-deploy-helper  | user    | CRITICAL    | Broken symlink                  |
| marketing-copy-v2  | user    | DUPLICATE?  | 72% overlap with marketing-copy |
```

### Recommended Actions
For each issue found, suggest:
- Broken symlinks: `/skillscope-fix --prune`
- Quality issues: `/skillscope-fix`
- Duplicates: manual review, consider removing one
- Token waste: `/skillscope-tokens`

## Related Skills

- For the integrated visual dashboard: `skillscope dashboard --open`
- For inventory only: `/skillscope-audit`
- For auto-fixing: `/skillscope-fix`
- For usage analytics: `/skillscope-usage`
- For token cost: `/skillscope-tokens`

## Dashboard Troubleshooting

Use `skillscope dashboard --open` for browser reports. Only add `--weeks N` when the user explicitly asks for a custom analysis period.

If a dashboard command fails with `unexpected argument '--weeks'`, retry once without `--weeks` and report that the installed CLI is stale; users should update to `skillscope` 2.0.1 or newer.

If it fails with `Dashboard template not found`, tell the user to install or update to the embedded-template CLI build (`skillscope` 2.0.1 or newer). Raw `scan --json` is a fallback, not the main dashboard path.
