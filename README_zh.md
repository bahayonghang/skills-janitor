# Skillscope

[English](README.md) | [中文](README_zh.md)

审计、追踪使用情况、检查健康状态，并为 Claude Code 与 OpenAI Codex skills 打开本地 HTML dashboard。

## CLI 安装

用 Cargo 安装 `skillscope` CLI：

```bash
cargo install skillscope --git https://github.com/bahayonghang/skillscope --bin skillscope --locked --force
skillscope --version
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
| `/skillscope-audit` | 查看所有已安装技能的完整清单。 |
| `/skillscope-report` | 检查 lint 问题、重复技能、损坏技能和维护建议。 |
| `/skillscope-fix` | 预览或应用安全修复，配合 `--prune` 处理损坏或孤立技能。 |
| `/skillscope-usage` | 查看哪些技能常用，哪些技能闲置。 |
| `/skillscope-tokens` | 估算每个技能占用的上下文窗口 token 成本。 |
| `/skillscope-search` | 在 GitHub 搜索技能，或把本地技能与替代方案对比。 |
| `/skillscope-precheck` | 安装新技能前检查它是否与已有技能重叠。 |

## 核心使用示例

```bash
/skillscope-audit
/skillscope-report
/skillscope-usage
/skillscope-tokens
/skillscope-audit "打开 dashboard"
/skillscope-search n8n
/skillscope-search --compare my-skill
/skillscope-precheck https://github.com/user/repo/tree/main/skills/my-skill
/skillscope-fix
/skillscope-fix --prune
/skillscope-fix --apply
/skillscope-fix --prune --apply
```

Dashboard 是本地 HTML 使用情况与健康度看板，支持 EN / 中文切换：

```bash
skillscope dashboard --open
```

## 自然语言示例

```text
"审计我的技能"
"给我的技能做一次健康检查"
"我用了哪些技能？"
"我的技能消耗多少 token？"
"打开 Skillscope dashboard"
"搜索 n8n 技能"
"对比我的 deploy-helper 技能"
"安装前检查这个技能"
"预览损坏技能的修复"
```

## 安全说明

- `/skillscope-fix` 默认是 dry-run，只预览不写入。
- 只有明确加上 `--apply` 时，CLI 才会写入变更。
- 用 `/skillscope-fix --prune` 先预览损坏符号链接、空技能目录或孤立技能。
- 只有确认要执行这些清理动作时，才使用 `/skillscope-fix --prune --apply`。
- fix 操作会跳过 plugin 和 marketplace 技能，因为更新可能覆盖本地修改。

## 致谢

Skillscope 基于 MIT 许可的原始项目 [khendzel/skills-janitor](https://github.com/khendzel/skills-janitor) 构建。本 fork 已扩展为跨平台 Rust CLI，覆盖 Claude Code 与 Codex skill 审计、usage/token 分析、健康检查、内置 skills 和嵌入式 HTML dashboard。

## 许可证

MIT
