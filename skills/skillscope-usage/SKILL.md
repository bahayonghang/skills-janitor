---
name: skillscope-usage
description: "Show which skills you use and which you never use. Use when the user asks about skill usage, unused skills, or wants to know which skills are active."
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
# Usage Tracking

Parse your Claude Code conversation history to see which skills you actually invoke and which are dead weight.

## How to Run

```bash
skillscope usage [--weeks N] [--json]
```

Legacy fallback only when explicitly requested:

```bash
bash <scripts_dir>/usage.sh [--weeks N] [--json]
```

- `--weeks N` - how many weeks to analyze (default: 4)
- `--json` - output raw JSON instead of formatted table

## What It Detects

### Explicit Invocations
Slash commands starting with `/skill-name` (e.g., `/n8n-workflows`, `/skillscope-audit`). Most reliable signal.

### Estimated Invocations
Natural language matching against skill description keywords. Higher threshold (50%) to avoid false positives. Labeled as "estimated" in output.

## Example Output

```
=== Skillscope - Usage Report ===
Period: 2026-02-24 to 2026-03-24 (4 weeks)

--- Most Used ---
  Skill                    Explicit  Estimated  Total
  n8n-workflows                   2          0      2
  23studio-social-post            1          0      1

--- Never Used (32 skills) ---
  marketing-ab-test        (user)
  marketing-analytics      (user)
  ... and 30 more

=== Summary ===
  Active skills: 4 / 36 (11%)
  Unused skills: 32 (89%)
```

## Persistent Data

Results are saved to `data/usage-history.json`, keeping the last 12 weeks for trend tracking across runs.

## Related Skills

- For the integrated visual dashboard: `skillscope dashboard --open`
- For finding better alternatives: `/skillscope-search`
- For comparing against the market: `/skillscope-search --compare`
- For removing unused skills: `/skillscope-fix --prune`

## Dashboard Troubleshooting

Use `skillscope dashboard --open` for browser reports. Only add `--weeks N` when the user explicitly asks for a custom analysis period.

If a dashboard command fails with `unexpected argument '--weeks'`, retry once without `--weeks` and report that the installed CLI is stale; users should update to `skillscope` 2.0.1 or newer.

If it fails with `Dashboard template not found`, tell the user to install or update to the embedded-template CLI build (`skillscope` 2.0.1 or newer). Raw `scan --json` is a fallback, not the main dashboard path.
