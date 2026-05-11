---
name: janitor-search
description: "Search GitHub for new skills to install. Also use with --compare to analyze a local skill against GitHub alternatives and marketplace competition."
metadata:
  version: 2.0.0
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
# Skill Discovery & Comparison

Search GitHub for Claude Code skills by keyword, or compare a local skill against alternatives.

## How to Run

```bash
# Search for skills
skills-janitor search <keyword> [--limit N] [--json]

# Compare your skill against GitHub alternatives
skills-janitor compare <skill-name> [--json]
```

Legacy fallback only when explicitly requested:

```bash
bash <scripts_dir>/search.sh <keyword> [--limit N] [--json]
bash <scripts_dir>/search.sh --compare <skill-name> [--json]
```

## Search Mode

- `<keyword>` - required search term (e.g., "marketing", "deployment", "testing")
- `--limit N` - max results (default: 10)
- `--json` - raw JSON output

### How It Works

1. Searches GitHub repos matching `{keyword} claude skill` in name/description/README
2. Also searches by topic tag `claude-code`
3. Cross-references results against your installed skills and plugins
4. Marks each result as `INSTALLED` or `AVAILABLE`

## Compare Mode (`--compare`)

Analyzes a local skill against alternatives found on GitHub:
- Composite scoring: 40% keyword overlap, 30% popularity, 15% recency, 15% activity
- Shows marketplace install counts when available
- Reports market position (unique niche vs. crowded space)

```bash
skills-janitor compare my-marketing-skill
```

## Rate Limits

- Unauthenticated: 60 requests/hour
- Set `GITHUB_TOKEN` env var for 5,000 requests/hour
- Results cached for 24 hours

## Related Skills

- For checking which skills you actually use: `/janitor-usage`
- For token cost analysis: `/janitor-tokens`
- For pre-install overlap check: `/janitor-precheck`
