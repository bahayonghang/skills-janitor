# Skills Janitor (Windows Edition)

[English](README.md) | [中文](README_zh.md)

> Audit, track usage, and manage your AI coding skills - 7 focused skills, zero dependencies.

This is a fork of [khendzel/skills-janitor](https://github.com/khendzel/skills-janitor). Thanks to Krzysztof Hendzel and the original contributors for the excellent foundation.

**本仓库目标：在上游仓库的基础上添加对 Windows 的核心支持。** 包括路径兼容、shell 行为适配、Windows 原生环境下的完整功能运行。

Works with **Claude Code** and **OpenAI Codex**.

![Skills Janitor](demo.gif)

A plugin that keeps your skills ecosystem clean, organized, and healthy. Automatically detects and scans skills from both Claude Code (`~/.claude/skills/`) and OpenAI Codex (`~/.agents/skills/`).

## What's New in v1.2

Correctness release — bundled five fixes that resolve real data-loss and noise problems found in the wild.

- **`fix.sh --apply` is safe again** — recognizes nested `metadata.version` (the canonical layout). Previously would have injected duplicate top-level `version:` lines into every modern skill.
- **Name-collision detection** — `janitor-report` now flags skills with the same name living at different real paths (the situation that confuses skill triggering), not just description-similarity overlaps.
- **Symlink shadows no longer reported as duplicates** — the same physical SKILL.md reachable from `~/.claude/skills/` and `~/.agents/skills/` is counted once.
- **`tokencost` reflects actual cost** — no more double-counted symlinks inflating the wasted-budget number.
- **Plugin info populated** — `installed_plugins.json` parser updated for Claude Code v2 schema (was always reporting 0 plugins).

Plus prior unreleased lint fixes (Windows compatibility, multiline descriptions, support-folder false-positives).

### What's still in v1.1

- **Cross-platform** — works with both Claude Code and OpenAI Codex
- **Pre-install overlap check** — `/janitor-precheck` checks if a new skill duplicates existing ones before installing
- **Context window token cost** — `/janitor-tokens` shows per-skill token consumption
- **Consolidated from 9 to 7 commands** — fewer commands, same coverage

## Skills

| Command | What it does |
|---------|-------------|
| `/janitor-audit` | Full inventory of all installed skills |
| `/janitor-report` | Health check: lint, duplicates, broken skills, recommendations |
| `/janitor-fix` | Auto-fix issues + `--prune` to remove broken skills |
| `/janitor-usage` | Track which skills you use and which you never use |
| `/janitor-tokens` | Show context window token cost per skill |
| `/janitor-search` | Search GitHub for skills + `--compare` for market analysis |
| `/janitor-precheck` | Check overlap before installing a new skill |

## Install

**Plugin install (recommended):**
```
/plugin marketplace add khendzel/skills-janitor
/plugin install skills-janitor
```

**Or clone directly:**
```bash
git clone https://github.com/khendzel/skills-janitor ~/.claude/skills/skills-janitor
```

## Rust CLI

Skills Janitor now ships a cross-platform Rust CLI named `skills-janitor`. The slash-command skills call this CLI first, so normal operation no longer depends on Python, Bash, or curl.

Install from this repository with Cargo:

```bash
cargo install skills-janitor --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
```

For local development:

```bash
just ci
just install-local
skills-janitor --version
```

Core commands:

```bash
skills-janitor scan --json
skills-janitor report
skills-janitor fix [--apply] [--prune]
skills-janitor usage [--weeks N] [--json]
skills-janitor tokens [--budget N] [--weeks N] [--json]
skills-janitor search <keyword> [--limit N] [--json]
skills-janitor compare <skill-name> [--json]
skills-janitor precheck <github-url-or-path> [--json]
skills-janitor dashboard [--open] [--output path] [--weeks N] [--budget N]
```

The dashboard is a static, self-contained HTML audit view. It highlights Critical/Warning issues, unused and low-frequency skills, token waste, duplicate overlap, usage frequency, inventory, plugins, commands, and the latest 20 embedded snapshots:

```bash
skills-janitor dashboard --open
skills-janitor dashboard --output target/tmp/janitor-dashboard.html --weeks 52 --budget 200000
```

GitHub Releases publish prebuilt archives for Windows, Linux, and macOS. Use those binaries if you do not have Rust installed.
## Usage examples

Each skill has its own slash command with autocomplete:

```
/janitor-audit          -> full skill inventory
/janitor-report         -> health check (lint + duplicates + broken)
/janitor-usage          -> which skills you actually invoke
/janitor-tokens         -> context window cost per skill
/janitor-audit "open dashboard" -> visual HTML dashboard
/janitor-search         -> find skills on GitHub
/janitor-search --compare my-skill  -> market analysis vs alternatives
/janitor-precheck https://github.com/user/skill  -> check before installing
/janitor-fix            -> auto-fix (dry-run by default)
/janitor-fix --prune    -> find and remove broken skills
```

Or use natural language:
```
"audit my skills"
"which skills do I use?"
"how many tokens do my skills cost?"
"check this skill before installing"
"search for n8n skills"
```

## Usage tracking

Parses your conversation history to show which skills you invoke and which you never use:

```
=== Skills Janitor - Usage Report ===
Period: 2026-02-24 to 2026-03-23 (4 weeks)

Active skills: 4 / 36 (11%)
Unused skills: 32 (89%)
Most used: n8n-workflows (17 total)
Recommendation: Remove 32 unused skills
```

## Token cost analysis

Shows how many context window tokens each skill consumes:

```
=== Skills Janitor - Context Window Cost ===
Budget: 200,000 tokens

  Skill                               Tokens  Budget  Used?
  marketing-copywriting                2,340   1.2%    yes
  marketing-seo-audit                  1,560   0.8%     NO
  ...

  Total token cost: 18,720 (9.4% of budget)
  Unused skill cost: 14,300 (7.2% of budget wasted)
```

## Pre-install check

Check if a new skill overlaps with existing ones before installing:

```
=== Skills Janitor - Pre-Install Check ===

  Checking: marketing-seo-v2
  Scanned 35 installed skills

  --- HIGH OVERLAP (likely duplicates) ---
    [72%] marketing-seo-audit (user)
         Shared: seo, audit, ranking, meta

  VERDICT: High overlap detected - likely duplicate
```

## Duplicate detection (new in v1.2)

`/janitor-report` now flags two distinct kinds of duplicates separately:

```
=== Skills Janitor - Duplicate Detection ===

Total skill records: 73
Unique skill files (after symlink dedup): 60

--- Name Collisions ---
Found 2 skill name(s) at multiple distinct paths:

  baseline-ui
    [user] ~/.claude-account-personal/skills/baseline-ui
    [user] ~/.agents/skills/baseline-ui

  fixing-accessibility
    [user] ~/.claude-account-personal/skills/fixing-accessibility
    [user] ~/.agents/skills/fixing-accessibility

--- Description Overlap (Jaccard > 30%) ---
Found 3 potential overlap(s):

  [50%] janitor-audit <-> janitor-usage
       Scopes: user / user
       Shared keywords: show, skills

  [33%] n8n-code-javascript <-> n8n-code-python
       Scopes: user / user
       Shared keywords: code, input, json, node, nodes, syntax
```

**Name collisions** are exact-name conflicts at different real paths — the situation that makes Claude pick the wrong skill ambiguously. **Description overlaps** are semantic-similarity warnings for skills with overlapping triggers but different names.

Symlink shadows (the same physical SKILL.md reachable from both `~/.claude/skills/` and `~/.agents/skills/`) are deduped via `realpath` and no longer counted as duplicates.

## What it won't do

- Never deletes anything without explicit confirmation
- Never modifies plugin/marketplace skills
- Respects that some overlap is intentional
- Dry-run by default for all destructive operations

## Supported Platforms

| Platform | User skills | Project skills | Usage tracking |
|----------|-------------|----------------|----------------|
| Claude Code | `~/.claude/skills/` | `./.claude/skills/` | Full (history.jsonl) |
| OpenAI Codex | `~/.agents/skills/` | `./.agents/skills/` | Keyword matching |

Skills Janitor auto-detects which platforms are installed and scans all of them.

## Requirements

- Rust CLI: `skills-janitor`
- Runtime: no Python, Bash, curl, pip installs, or node modules required
- Legacy `scripts/*.sh` remain as explicit fallback/compatibility helpers

## Structure

```
skills-janitor/
├── .claude-plugin/
│   └── marketplace.json      # Plugin manifest (7 skills)
├── skills/
│   ├── janitor-audit/SKILL.md
│   ├── janitor-report/SKILL.md
│   ├── janitor-fix/SKILL.md
│   ├── janitor-usage/SKILL.md
│   ├── janitor-tokens/SKILL.md
│   ├── janitor-search/SKILL.md
│   └── janitor-precheck/SKILL.md
├── Cargo.toml                # installable CLI shim and workspace
├── cli/                      # Rust CLI crate
├── justfile                  # fmt/clippy/test/build/ci entry points
├── .github/workflows/        # CI and release binary workflows
├── scripts/                  # Legacy bash+python fallback scripts
├── demo.gif
├── LICENSE                   # MIT
└── README.md
```

## Migrating from v1.0

If you used v1.0 (9 skills), here's what changed:

| Old command | New equivalent |
|-------------|---------------|
| `/janitor-check` | `/janitor-report` (includes lint checks) |
| `/janitor-duplicates` | `/janitor-report` (includes duplicate detection) |
| `/janitor-cleanup` | `/janitor-fix --prune` |
| `/janitor-compare` | `/janitor-search --compare <name>` |

## Contributing

PRs welcome. Run `just ci` before sending changes. Each skill is self-contained in `skills/janitor-*/SKILL.md`, while shared runtime behavior lives in the Rust CLI under `cli/`.

## License

MIT
