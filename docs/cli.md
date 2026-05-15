# CLI 使用方法

`skillscope` 是 Skillscope 的跨平台 Rust CLI。所有内置 Skillscope skills 都会优先调用它，因此 CLI 是最稳定、最适合自动化的入口。

## 安装与验证

从当前仓库安装：

```bash
cargo install skillscope --git https://github.com/bahayonghang/skillscope --bin skillscope --locked --force
```

验证安装：

```bash
skillscope --version
```

本地开发安装：

```bash
just install
skillscope --version
```

`just install` 会先通过 `cargo install --path . --locked --force` 安装 exe，然后把仓库 `skills/*` 复制到当前项目的 `.claude/skills/` 和 `.agents/skills/`。如果要额外测试其他项目级 skill 目录，可以追加目标：

```bash
just install --target kiro
```

额外 `--target <name>` 会解析为 `.<name>/skills/`，例如 `--target kiro` 会同步到 `.kiro/skills/`。`just install-local` 是同一安装流程的别名，也支持相同参数。

## 扫描范围

CLI 会自动检测可用平台，并扫描存在的技能目录：

| 平台 | 用户级目录 | 项目级目录 | scope |
| --- | --- | --- | --- |
| Claude Code | `~/.claude/skills/` | `./.claude/skills/` | `user` / `project` |
| OpenAI Codex | `~/.agents/skills/` | `./.agents/skills/` | `codex-user` / `codex-project` |

Claude Code 的命令和插件信息也会被读取：

- 用户命令：`~/.claude/commands/`
- 项目命令：`./.claude/commands/`
- 插件信息：`~/.claude/plugins/installed_plugins.json`

## 命令总览

```bash
skillscope scan --json
skillscope report [--json]
skillscope fix [--apply] [--dry-run] [--prune] [--json]
skillscope usage [--weeks N] [--json]
skillscope tokens [--budget N] [--weeks N] [--json]
skillscope search <keyword> [--limit N] [--json]
skillscope compare <skill-name> [--json]
skillscope precheck <github-url-or-path> [--json]
skillscope dashboard [--open] [--output path] [--weeks N] [--budget N]
```

多数命令支持 `--json`，适合脚本、CI 或其他工具读取。

## `scan`：生成技能清单

扫描所有已安装技能，并输出每个技能的基础信息。

```bash
skillscope scan
skillscope scan --json
```

扫描记录包含：

- 文件夹名、scope、平台和路径。
- 真实路径、是否符号链接、符号链接目标。
- 是否存在 `SKILL.md` 或 `Skill.md`。
- frontmatter 中的 `name`、`description`、`metadata.version`。
- 是否有 frontmatter、正文、行数和额外文件数量。
- 已安装插件、Claude commands 和损坏符号链接数量。

适合用于资产盘点、导出 inventory、在 CI 中检查技能目录是否可读。

## `report`：健康检查

运行 lint 与重复检测，给出技能健康报告。

```bash
skillscope report
skillscope report --json
```

它会报告三类 lint 严重性：

| 严重性 | 示例问题 |
| --- | --- |
| Critical | 损坏符号链接、缺少 `SKILL.md`、缺少 frontmatter、缺少 `description` |
| Warning | 缺少 `name`、描述过短、描述没有说明触发时机、正文内容过少 |
| Info | 文件夹名与技能名不一致、描述较长、缺少 Gotchas section、技能文件过大 |

重复检测包含：

- 名称冲突：同名技能存在于多个不同真实路径。
- 描述重叠：根据关键词相似度找出触发范围可能重叠的技能。

推荐把 `report` 作为常规维护入口，因为它同时覆盖结构质量和重复风险。

## `fix`：预览或应用修复

`fix` 默认是 dry-run，只显示计划变更，不写入文件。

```bash
skillscope fix
skillscope fix --json
```

应用可修复问题：

```bash
skillscope fix --apply
```

可自动处理的问题包括：

- 为缺少 delimiter 的 frontmatter 补 `---`。
- 为缺失或空的 `description` 填入模板描述。
- 在没有版本信息时添加 `metadata.version: "1.0.0"`。

安全边界：

- 默认不写入，必须加 `--apply`。
- 跳过插件和 marketplace 技能。
- 损坏符号链接默认不修复，交给 `--prune` 检查。
- 对存在 `metadata:` 但缺少 `version` 的复杂情况，只提示手动补充，避免破坏结构。

### prune 模式

查找可清理的损坏技能或空目录：

```bash
skillscope fix --prune
skillscope fix --prune --apply
```

`--prune` 会检查：

- 指向已删除目标的损坏符号链接。
- 没有 `SKILL.md` 且为空的技能目录。
- 没有 `SKILL.md` 但仍包含文件的目录会被跳过并提示人工检查。

## `usage`：使用频率分析

分析最近一段时间的对话历史，统计哪些技能被调用过。

```bash
skillscope usage
skillscope usage --weeks 12
skillscope usage --weeks 52 --json
```

参数：

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `--weeks N` | `4` | 分析最近 N 周的历史 |
| `--json` | 关闭 | 输出机器可读 JSON |

它会读取 Claude Code 历史文件和项目会话 JSONL，主要识别 `/skill-name` 和 `"skill-name"` 形式的明确调用。结果会持久化到：

```text
~/.claude/skills/skillscope/data/usage-history.json
```

最多保留最近 12 次使用报告，便于观察趋势。

## `tokens`：上下文 token 成本

估算每个技能在上下文窗口中占用的 token，并结合 usage 结果标记未使用技能。

```bash
skillscope tokens
skillscope tokens --budget 200000 --weeks 12
skillscope tokens --json
```

参数：

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `--budget N` | `200000` | 用于计算百分比的上下文预算 |
| `--weeks N` | `4` | 关联最近 N 周使用记录 |
| `--json` | 关闭 | 输出机器可读 JSON |

估算方式：

- 读取每个技能的 `SKILL.md` 内容。
- ASCII 文本按约 4 字符一个 token 估算。
- CJK 字符按单字符计入。
- 通过真实路径去重，避免同一文件被 Claude 和 Codex 双目录引用时重复计费。

常见用法是先看未使用且 token 成本高的技能，再用 `report` 检查质量问题，最后手动删除或合并。

## `search`：搜索 GitHub 技能

按关键词搜索 GitHub 上包含 `SKILL.md` 的代码结果。

```bash
skillscope search deployment
skillscope search marketing --limit 20
skillscope search testing --json
```

参数：

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `<keyword>` | 必填 | 搜索关键词 |
| `--limit N` | `10` | 最多返回结果数，内部上限为 50 |
| `--json` | 关闭 | 输出机器可读 JSON |

GitHub API 未认证时有较低速率限制。需要更高额度时设置：

```bash
export GITHUB_TOKEN=...
```

PowerShell：

```powershell
$env:GITHUB_TOKEN = "..."
```

## `compare`：市场对比

根据本地已安装技能的描述关键词，在 GitHub 搜索替代方案。

```bash
skillscope compare my-marketing-skill
skillscope compare deploy-helper --json
```

工作流程：

1. 在本地技能目录中查找 `<skill-name>`。
2. 读取该技能的 frontmatter description。
3. 从描述中提取关键词。
4. 用前几个关键词搜索 GitHub 替代实现。

如果技能不存在，会直接报错。

## `precheck`：安装前重叠检查

在安装新技能前，检查它是否和已有技能高度重叠。

```bash
skillscope precheck https://github.com/user/repo/tree/main/skills/my-skill
skillscope precheck C:\Users\me\skills\my-skill
skillscope precheck ./local-skill --json
```

支持输入：

- GitHub skill 文件夹 URL。
- GitHub repo root，仓库根目录下有 `SKILL.md`。
- GitHub raw `SKILL.md` URL。
- 本地技能目录或本地 `SKILL.md` 文件。

判定规则：

| 最高重叠度 | 结论 |
| --- | --- |
| `>= 60%` | `HIGH_OVERLAP`，很可能重复 |
| `>= 30%` | `MODERATE_OVERLAP`，安装前应人工确认 |
| `< 30%` | `SAFE`，未发现显著重叠 |

## `dashboard`：生成本地使用情况看板

生成或更新自包含 HTML dashboard。当前页面重点展示技能使用覆盖率、未使用 / 低频技能、健康分、lint 问题、重复项、插件和命令信息，并支持 EN / 中文切换。

```bash
skillscope dashboard
skillscope dashboard --open --weeks 52
skillscope dashboard --output target/tmp/skillscope-dashboard.html --weeks 52 --budget 200000
```

参数：

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `--open` | 关闭 | 生成后用默认浏览器打开 |
| `--output path` | `./skillscope-dashboard.html` | 输出 HTML 路径 |
| `--weeks N` | `52` | dashboard 中 usage 的分析周期 |
| `--budget N` | `200000` | 兼容旧 snapshot 的 token 预算参数；当前 dashboard 不展示 token 成本模块 |

dashboard 会追加快照并保留最近 20 次。页面围绕 inventory、usage、lint、duplicate、plugins / commands 和健康分组织信息，适合周期性维护；旧 snapshot 中的 tokens 数据仍可兼容读取，但不再作为可视主模块。

## 推荐工作流

### 日常体检

```bash
skillscope report
skillscope usage --weeks 12
skillscope tokens --budget 200000 --weeks 12
```

### 清理前评估

```bash
skillscope dashboard --open --weeks 52
skillscope fix --prune
```

确认输出后再执行：

```bash
skillscope fix --prune --apply
```

### 安装新技能前

```bash
skillscope precheck https://github.com/user/repo/tree/main/skills/example
```

如发现中高重叠，先用 `report` 检查已有技能，再决定是否安装、合并或放弃。

## 本仓库开发命令

仓库根目录提供 `justfile`：

```bash
just fmt
just clippy
just test
just build
just ci
just install
just run report
```

发布或提交前建议运行：

```bash
just ci
```
