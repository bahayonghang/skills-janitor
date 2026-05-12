# Skills Janitor (Windows Edition)

[English](README.md) | [中文](README_zh.md)

Audit, track usage, and clean up your AI coding skills with seven focused janitor commands.

This fork of [khendzel/skills-janitor](https://github.com/khendzel/skills-janitor) adds core Windows support, including path handling, shell behavior, and native Windows operation. It works with **Claude Code** and **OpenAI Codex** skills.

## CLI Installation

Install the `skills-janitor` CLI with Cargo:

```bash
cargo install skills-janitor --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
skills-janitor --version
```

If you do not have Rust/Cargo installed, download the prebuilt binary for your platform from GitHub Releases.

## Skills Overview

| Command | Use it for |
|---------|------------|
| `/janitor-audit` | Show the full installed skill inventory. |
| `/janitor-report` | Run a health check for lint issues, duplicates, broken skills, and recommendations. |
| `/janitor-fix` | Preview or apply safe fixes, with `--prune` for broken or orphaned skills. |
| `/janitor-usage` | See which skills you use and which ones are idle. |
| `/janitor-tokens` | Estimate context-window token cost per skill. |
| `/janitor-search` | Search GitHub for skills, or compare a local skill with alternatives. |
| `/janitor-precheck` | Check whether a new skill overlaps with installed skills before adding it. |

## Core Usage Examples

```bash
/janitor-audit
/janitor-report
/janitor-usage
/janitor-tokens
/janitor-audit "open dashboard"
/janitor-search n8n
/janitor-search --compare my-skill
/janitor-precheck https://github.com/user/repo/tree/main/skills/my-skill
/janitor-fix
/janitor-fix --prune
/janitor-fix --apply
/janitor-fix --prune --apply
```

The dashboard is a local HTML audit view:

```bash
skills-janitor dashboard --open
```

## Natural Language Examples

```text
"audit my skills"
"run a health check on my skills"
"which skills do I use?"
"how many tokens do my skills cost?"
"open the janitor dashboard"
"search for n8n skills"
"compare my deploy-helper skill"
"check this skill before installing it"
"preview fixes for broken skills"
```

## Safety Notes

- `/janitor-fix` runs as a dry-run by default.
- Add `--apply` only when you want the CLI to write changes.
- Use `/janitor-fix --prune` to preview broken symlinks, empty skill folders, or orphaned skills before removal.
- Use `/janitor-fix --prune --apply` only when you want those prune actions applied.
- Plugin and marketplace skills are skipped by fix operations because updates can overwrite local edits.

## License

MIT
