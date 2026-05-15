# Skills Janitor（Windows 版）

[English](README.md) | [中文](README_zh.md)

用 7 个专注的 janitor 命令审计、追踪和清理你的 AI 编程技能。

本仓库 fork 自 [khendzel/skills-janitor](https://github.com/khendzel/skills-janitor)，在上游基础上加入 Windows 核心支持，包括路径兼容、shell 行为适配和 Windows 原生环境运行。兼容 **Claude Code** 和 **OpenAI Codex** 技能。

## CLI 安装

用 Cargo 安装 `skills-janitor` CLI：

```bash
cargo install skills-janitor --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
skills-janitor --version
```

在本仓库做本地开发时，可以同时安装 CLI 和内置 skills：

```bash
just install
# 额外复制到 .kiro/skills 等项目目标
just install --target kiro
```

`just install` 默认把 `skills/*` 复制到当前项目的 `.claude/skills/` 和 `.agents/skills/` 目录。额外的 `--target <name>` 会解析为 `.<name>/skills/`。

如果没有安装 Rust/Cargo，可以从 GitHub Releases 下载对应平台的预编译二进制。

## Skills 概览

| 命令 | 用途 |
|------|------|
| `/janitor-audit` | 查看所有已安装技能的完整清单。 |
| `/janitor-report` | 检查 lint 问题、重复技能、损坏技能和维护建议。 |
| `/janitor-fix` | 预览或应用安全修复，配合 `--prune` 处理损坏或孤立技能。 |
| `/janitor-usage` | 查看哪些技能常用，哪些技能闲置。 |
| `/janitor-tokens` | 估算每个技能占用的上下文窗口 token 成本。 |
| `/janitor-search` | 在 GitHub 搜索技能，或把本地技能与替代方案对比。 |
| `/janitor-precheck` | 安装新技能前检查它是否与已有技能重叠。 |

## 核心使用示例

```bash
/janitor-audit
/janitor-report
/janitor-usage
/janitor-tokens
/janitor-audit "打开 dashboard"
/janitor-search n8n
/janitor-search --compare my-skill
/janitor-precheck https://github.com/user/repo/tree/main/skills/my-skill
/janitor-fix
/janitor-fix --prune
/janitor-fix --apply
/janitor-fix --prune --apply
```

Dashboard 是本地 HTML 审计视图：

```bash
skills-janitor dashboard --open
```

## 自然语言示例

```text
"审计我的技能"
"给我的技能做一次健康检查"
"我用了哪些技能？"
"我的技能消耗多少 token？"
"打开 janitor dashboard"
"搜索 n8n 技能"
"对比我的 deploy-helper 技能"
"安装前检查这个技能"
"预览损坏技能的修复"
```

## 安全说明

- `/janitor-fix` 默认是 dry-run，只预览不写入。
- 只有明确加上 `--apply` 时，CLI 才会写入变更。
- 用 `/janitor-fix --prune` 先预览损坏符号链接、空技能目录或孤立技能。
- 只有确认要执行这些清理动作时，才使用 `/janitor-fix --prune --apply`。
- fix 操作会跳过 plugin 和 marketplace 技能，因为更新可能覆盖本地修改。

## 许可证

MIT
