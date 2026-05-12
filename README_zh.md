# Skills Janitor (Windows 版)

[English](README.md) | [中文](README_zh.md)

> 审计、追踪使用情况、管理你的 AI 编程技能 —— 7 个专注技能，零依赖。

本仓库 fork 自 [khendzel/skills-janitor](https://github.com/khendzel/skills-janitor)。感谢 Krzysztof Hendzel 和原作者们打下的优秀基础。

**本仓库目标：在上游仓库的基础上添加对 Windows 的核心支持。** 包括路径兼容、shell 行为适配、Windows 原生环境下的完整功能运行。

兼容 **Claude Code** 和 **OpenAI Codex**。

![Skills Janitor](demo.gif)

一个保持技能生态干净、有序、健康的插件。自动检测并扫描 Claude Code（`~/.claude/skills/`）和 OpenAI Codex（`~/.agents/skills/`）中的技能。

## v1.2 更新

正确性修复版本 —— 修复了五个在实际使用中发现的数据丢失和噪声问题。

- **`fix.sh --apply` 恢复安全** —— 识别嵌套的 `metadata.version`（标准布局）。之前会向每个现代技能注入重复的顶层 `version:` 行。
- **名称冲突检测** —— `janitor-report` 现在会标记同名但位于不同真实路径的技能（导致技能触发混淆的情况），不再仅限于描述相似度重叠。
- **符号链接阴影不再报告为重复** —— 从 `~/.claude/skills/` 和 `~/.agents/skills/` 都能访问到的同一个物理 SKILL.md 只计数一次。
- **`tokencost` 反映真实成本** —— 不再因符号链接重复计数而虚增浪费的预算数字。
- **插件信息已填充** —— `installed_plugins.json` 解析器已更新适配 Claude Code v2 schema（之前始终报告 0 个插件）。

另外包含之前未发布的 lint 修复（Windows 兼容性、多行描述、support 文件夹误报）。

### v1.1 中已有功能

- **跨平台** —— 同时支持 Claude Code 和 OpenAI Codex
- **安装前重叠检查** —— `/janitor-precheck` 在安装新技能前检查是否与已有技能重复
- **上下文窗口 token 成本** —— `/janitor-tokens` 显示每个技能的 token 消耗
- **从 9 个命令整合为 7 个** —— 更少的命令，同样的覆盖

## 技能列表

| 命令 | 功能 |
|------|------|
| `/janitor-audit` | 所有已安装技能的完整清单 |
| `/janitor-report` | 健康检查：lint、重复、损坏技能、建议 |
| `/janitor-fix` | 自动修复 + `--prune` 移除损坏技能 |
| `/janitor-usage` | 追踪你使用了哪些技能、哪些从未使用 |
| `/janitor-tokens` | 显示每个技能的上下文窗口 token 成本 |
| `/janitor-search` | 在 GitHub 搜索技能 + `--compare` 市场分析 |
| `/janitor-precheck` | 安装新技能前检查重叠 |

## 安装

**插件安装（推荐）：**
```
/plugin marketplace add bahayonghang/skills-janitor
/plugin install skills-janitor
```

**或直接克隆：**
```bash
git clone https://github.com/bahayonghang/skills-janitor ~/.claude/skills/skills-janitor
```

## Rust CLI

Skills Janitor 现在提供跨平台 Rust CLI：`skills-janitor`。各 slash-command skill 会优先调用这个 CLI，因此正常运行不再依赖 Python、Bash 或 curl。

使用 Cargo 从本仓库安装：

```bash
cargo install skills-janitor --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
```

本地开发：

```bash
just ci
just install-local
skills-janitor --version
```

核心命令：

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

Dashboard 是一个静态、自包含的本地 HTML 审计视图。它会突出 Critical/Warning 问题、未使用和低频技能、token 浪费、重复/重叠、使用频率、完整清单、插件、命令，以及最近 20 次快照：

```bash
skills-janitor dashboard --open
skills-janitor dashboard --output target/tmp/janitor-dashboard.html --weeks 52 --budget 200000
```

GitHub Releases 会发布 Windows、Linux、macOS 预编译压缩包。没有 Rust 工具链时，直接下载对应平台二进制。
## 使用示例

每个技能都有带自动补全的斜杠命令：

```
/janitor-audit          -> 完整技能清单
/janitor-report         -> 健康检查（lint + 重复 + 损坏）
/janitor-usage          -> 你实际调用了哪些技能
/janitor-tokens         -> 每个技能的上下文窗口成本
/janitor-audit "打开 dashboard" -> 可视化 HTML 仪表盘
/janitor-search         -> 在 GitHub 搜索技能
/janitor-search --compare my-skill  -> 与替代方案的市场分析
/janitor-precheck https://github.com/user/skill  -> 安装前检查
/janitor-fix            -> 自动修复（默认 dry-run）
/janitor-fix --prune    -> 查找并移除损坏技能
```

也可以用自然语言：
```
"审计我的技能"
"我用了哪些技能"
"我的技能消耗多少 token"
"安装前检查这个技能"
"搜索 n8n 技能"
```

## 使用追踪

解析对话历史，展示你调用了哪些技能、哪些从未使用：

```
=== Skills Janitor - 使用报告 ===
时间范围: 2026-02-24 至 2026-03-23（4 周）

活跃技能: 4 / 36 (11%)
未使用技能: 32 (89%)
最常用: n8n-workflows（共 17 次）
建议: 移除 32 个未使用的技能
```

## Token 成本分析

显示每个技能消耗的上下文窗口 token 数：

```
=== Skills Janitor - 上下文窗口成本 ===
预算: 200,000 tokens

  技能                               Tokens  预算占比  使用?
  marketing-copywriting                2,340   1.2%    是
  marketing-seo-audit                  1,560   0.8%    否
  ...

  总 token 成本: 18,720（占预算 9.4%）
  未使用技能成本: 14,300（浪费预算 7.2%）
```

## 安装前检查

安装新技能前检查是否与已有技能重叠：

```
=== Skills Janitor - 安装前检查 ===

  检查: marketing-seo-v2
  扫描了 35 个已安装技能

  --- 高度重叠（可能是重复） ---
    [72%] marketing-seo-audit (user)
         共有关键词: seo, audit, ranking, meta

  结论: 检测到高度重叠 - 可能是重复技能
```

## 重复检测（v1.2 新增）

`/janitor-report` 现在分别标记两种不同类型的重复：

```
=== Skills Janitor - 重复检测 ===

总技能记录: 73
去重后唯一技能文件（符号链接去重后）: 60

--- 名称冲突 ---
发现 2 个技能名称存在于多个不同路径:

  baseline-ui
    [user] ~/.claude-account-personal/skills/baseline-ui
    [user] ~/.agents/skills/baseline-ui

  fixing-accessibility
    [user] ~/.claude-account-personal/skills/fixing-accessibility
    [user] ~/.agents/skills/fixing-accessibility

--- 描述重叠（Jaccard > 30%） ---
发现 3 个潜在重叠:

  [50%] janitor-audit <-> janitor-usage
       范围: user / user
       共有关键词: show, skills

  [33%] n8n-code-javascript <-> n8n-code-python
       范围: user / user
       共有关键词: code, input, json, node, nodes, syntax
```

**名称冲突**是不同真实路径上的同名技能 —— 会导致 Claude 模糊选择错误技能。**描述重叠**是触发词重叠但名称不同的技能的语义相似度警告。

符号链接阴影（从 `~/.claude/skills/` 和 `~/.agents/skills/` 都能访问的同一个物理 SKILL.md）通过 `realpath` 去重，不再计为重复。

## 不会做的事

- 未经明确确认不会删除任何内容
- 不会修改 plugin/marketplace 技能
- 尊重某些重叠是有意为之
- 所有破坏性操作默认 dry-run

## 支持的平台

| 平台 | 用户技能 | 项目技能 | 使用追踪 |
|------|----------|----------|----------|
| Claude Code | `~/.claude/skills/` | `./.claude/skills/` | 完整（history.jsonl） |
| OpenAI Codex | `~/.agents/skills/` | `./.agents/skills/` | 关键词匹配 |

Skills Janitor 自动检测已安装的平台并扫描全部。

## 依赖

- Rust CLI：`skills-janitor`
- 运行时：不需要 Python、Bash、curl、pip install 或 node modules
- 旧 `scripts/*.sh` 保留为显式 fallback/兼容脚本

## 项目结构

```
skills-janitor/
├── .claude-plugin/
│   └── marketplace.json      # 插件清单（7 个技能）
├── skills/
│   ├── janitor-audit/SKILL.md
│   ├── janitor-report/SKILL.md
│   ├── janitor-fix/SKILL.md
│   ├── janitor-usage/SKILL.md
│   ├── janitor-tokens/SKILL.md
│   ├── janitor-search/SKILL.md
│   └── janitor-precheck/SKILL.md
├── Cargo.toml                # 可安装 CLI shim 与 workspace
├── cli/                      # Rust CLI crate
├── justfile                  # fmt/clippy/test/build/ci 入口
├── .github/workflows/        # CI 与 release 二进制 workflow
├── scripts/                  # 旧 bash+python fallback 脚本
├── demo.gif
├── LICENSE                   # MIT
└── README.md
```

## 从 v1.0 迁移

如果你之前使用 v1.0（9 个技能），以下是变更对照：

| 旧命令 | 新等价命令 |
|--------|-----------|
| `/janitor-check` | `/janitor-report`（包含 lint 检查） |
| `/janitor-duplicates` | `/janitor-report`（包含重复检测） |
| `/janitor-cleanup` | `/janitor-fix --prune` |
| `/janitor-compare` | `/janitor-search --compare <name>` |

## 贡献

欢迎 PR。每个技能自包含在 `skills/janitor-*/SKILL.md` 中。

## 许可证

MIT
