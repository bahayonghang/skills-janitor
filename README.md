# Skillscope

[English](README.md) | [中文](README_zh.md)

Audit, track usage, check health, and open a local HTML dashboard for Claude Code and OpenAI Codex skills.

## CLI Installation

Install the `skillscope` CLI with Cargo:

```bash
cargo install skillscope --git https://github.com/bahayonghang/skillscope --bin skillscope --locked --force
skillscope --version
```

For local development from this repository, install the CLI and bundled skills together:

```bash
just install
# also copy skills into an extra project target such as .kiro/skills
just install --target kiro
```

`just install` copies `skills/*` into the current project's `.claude/skills/` and `.agents/skills/` directories by default. Extra `--target <name>` values resolve to `.<name>/skills/`.

If you do not have Rust/Cargo installed, download the prebuilt binary for your platform from GitHub Releases.

## Skills Overview

| Command | Use it for |
|---------|------------|
| `/skillscope-audit` | Show the full installed skill inventory. |
| `/skillscope-report` | Run a health check for lint issues, duplicates, broken skills, and recommendations. |
| `/skillscope-fix` | Preview or apply safe fixes, with `--prune` for broken or orphaned skills. |
| `/skillscope-usage` | See which skills you use and which ones are idle. |
| `/skillscope-tokens` | Estimate context-window token cost per skill. |
| `/skillscope-search` | Search GitHub for skills, or compare a local skill with alternatives. |
| `/skillscope-precheck` | Check whether a new skill overlaps with installed skills before adding it. |

## Core Usage Examples

```bash
/skillscope-audit
/skillscope-report
/skillscope-usage
/skillscope-tokens
/skillscope-audit "open dashboard"
/skillscope-search n8n
/skillscope-search --compare my-skill
/skillscope-precheck https://github.com/user/repo/tree/main/skills/my-skill
/skillscope-fix
/skillscope-fix --prune
/skillscope-fix --apply
/skillscope-fix --prune --apply
```

The dashboard is a local HTML usage-and-health view with an EN / 中文 language toggle:

```bash
skillscope dashboard --open
```

## Natural Language Examples

```text
"audit my skills"
"run a health check on my skills"
"which skills do I use?"
"how many tokens do my skills cost?"
"open the Skillscope dashboard"
"search for n8n skills"
"compare my deploy-helper skill"
"check this skill before installing it"
"preview fixes for broken skills"
```

## Safety Notes

- `/skillscope-fix` runs as a dry-run by default.
- Add `--apply` only when you want the CLI to write changes.
- Use `/skillscope-fix --prune` to preview broken symlinks, empty skill folders, or orphaned skills before removal.
- Use `/skillscope-fix --prune --apply` only when you want those prune actions applied.
- Plugin and marketplace skills are skipped by fix operations because updates can overwrite local edits.

## Acknowledgements

Skillscope builds on the original MIT-licensed [khendzel/skills-janitor](https://github.com/khendzel/skills-janitor) project. This fork has expanded into a cross-platform Rust CLI for Claude Code and Codex skill audit, usage and token analysis, health checks, bundled skills, and an embedded HTML dashboard.

## License

MIT
