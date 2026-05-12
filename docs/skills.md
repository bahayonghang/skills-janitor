# Skills 使用方法

Skills Janitor 提供 7 个 `janitor-*` 技能。它们适合在 AI 助手会话中使用：用户可以输入 slash command，也可以用自然语言描述目标，由助手选择合适的技能执行。

这些技能本身不重新实现扫描逻辑，而是先检查 `skills-janitor` Rust CLI 是否可用，再调用对应 CLI 命令。这样可以保持 Claude Code、OpenAI Codex、终端和 CI 的行为一致。

## 使用前提

每个 janitor skill 的第一步都会检查：

```bash
skills-janitor --version
```

如果 CLI 不存在，应先安装：

```bash
cargo install skills-janitor --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
```

没有 Rust/Cargo 时，使用 GitHub Releases 中的预编译二进制。除非用户明确要求 legacy mode，否则不建议回退到旧的 Bash/Python 脚本。

## 技能总览

| Skill | Slash command | 主要用途 | 底层 CLI |
| --- | --- | --- | --- |
| `janitor-audit` | `/janitor-audit` | 查看已安装技能清单 | `skills-janitor scan --json` |
| `janitor-report` | `/janitor-report` | 完整健康检查 | `skills-janitor report` |
| `janitor-fix` | `/janitor-fix` | 预览或应用自动修复 | `skills-janitor fix` |
| `janitor-usage` | `/janitor-usage` | 分析技能使用频率 | `skills-janitor usage` |
| `janitor-tokens` | `/janitor-tokens` | 估算上下文 token 成本 | `skills-janitor tokens` |
| `janitor-search` | `/janitor-search` | 搜索或对比 GitHub 技能 | `skills-janitor search` / `compare` |
| `janitor-precheck` | `/janitor-precheck` | 安装前重叠检查 | `skills-janitor precheck` |

## `/janitor-audit`：技能清单

用于查看当前安装了哪些技能，以及它们来自哪个 scope。

```text
/janitor-audit
```

适合在这些情况下使用：

- “列出我的技能”
- “审计已安装 skill”
- “看看 Claude 和 Codex 现在能加载哪些技能”

默认会调用：

```bash
skills-janitor scan --json
```

输出应整理成易读表格，通常包含：

- skill 名称。
- scope 和平台。
- 是否存在 `SKILL.md`。
- 关键 frontmatter 字段。
- symlink 是否有效。
- 是否有明显问题。

如果用户要求 HTML、可视化报告或 dashboard，优先使用：

```bash
skills-janitor dashboard --open --weeks 52
```

## `/janitor-report`：完整健康检查

用于一次性检查 lint、重复、损坏技能和维护建议。

```text
/janitor-report
```

底层命令：

```bash
skills-janitor report
```

适合在这些情况下使用：

- “检查我的技能有没有错误”
- “找重复 skill”
- “看看技能健康状态”
- “我想清理技能，先给我一份报告”

报告应优先展示严重问题：

1. Critical：缺失 `SKILL.md`、损坏 symlink、缺失 frontmatter 或 description。
2. Warning：描述过短、缺少触发时机、正文过少。
3. Info：命名不一致、描述过长、缺少 Gotchas、文件过大。
4. Duplicate：名称冲突和描述重叠。

后续动作建议：

- Critical 优先处理。
- 重复项需要人工确认，避免误删有意保留的技能。
- token 浪费和未使用技能可交给 `/janitor-tokens` 与 `/janitor-usage` 继续分析。

## `/janitor-fix`：自动修复

用于修复常见 metadata 问题。默认只是预览，不写入文件。

```text
/janitor-fix
/janitor-fix --apply
```

底层命令：

```bash
skills-janitor fix
skills-janitor fix --apply
```

它可以处理：

- 补齐 frontmatter delimiter。
- 给缺失或空 description 添加模板。
- 补齐 `metadata.version`。

安全规则：

- 默认 dry-run。
- `--apply` 才会写入。
- 跳过插件和 marketplace 技能。
- 不会自动处理需要人工判断的复杂 metadata 结构。

### `--prune` 清理模式

用于查找损坏 symlink、空技能目录等可删除对象。

```text
/janitor-fix --prune
/janitor-fix --prune --apply
```

底层命令：

```bash
skills-janitor fix --prune
skills-janitor fix --prune --apply
```

使用建议：

- 先运行不带 `--apply` 的 dry-run。
- 阅读输出，确认每个待删除对象确实无用。
- 再运行 `--apply`。

## `/janitor-usage`：使用情况

用于统计哪些技能实际被调用过，哪些长期未使用。

```text
/janitor-usage
/janitor-usage --weeks 12
```

底层命令：

```bash
skills-janitor usage --weeks 12
```

适合在这些情况下使用：

- “我用了哪些技能？”
- “哪些 skill 从没用过？”
- “清理前先看活跃技能”

结果通常包含：

- 分析周期。
- 活跃技能数量。
- 未使用技能数量。
- 最常用技能。
- 每个技能的调用次数和 scope。

由于 usage 依赖本地对话历史，Codex 与 Claude 的可观测程度可能不同。明确 slash command 调用最可靠，自然语言触发只能作为辅助信号。

## `/janitor-tokens`：token 成本

用于估算技能文件占用的上下文窗口成本。

```text
/janitor-tokens
/janitor-tokens --budget 200000 --weeks 12
```

底层命令：

```bash
skills-janitor tokens --budget 200000 --weeks 12
```

适合在这些情况下使用：

- “我的技能占多少 token？”
- “哪些未使用技能最浪费上下文？”
- “上下文预算被哪些技能吃掉了？”

建议解读方式：

- 先看 token 高且未使用的技能。
- 再看这些技能是否和其他技能重复。
- 如果只是大但经常使用，不应只因为体积大就删除。
- 如果长期未使用且和其他技能重叠，优先考虑合并或移除。

## `/janitor-search`：搜索和比较

用于在 GitHub 查找技能，或比较本地技能与外部替代方案。

```text
/janitor-search deployment
/janitor-search --compare my-skill
```

底层命令：

```bash
skills-janitor search deployment
skills-janitor compare my-skill
```

搜索模式适合：

- “找 n8n skills”
- “有没有 deployment 相关的 skill？”
- “帮我找测试相关 skill”

比较模式适合：

- “我的 skill 和 GitHub 上的替代方案比怎么样？”
- “这个 skill 是独特的，还是已有很多类似的？”

GitHub API 可能遇到速率限制。需要更高额度时设置 `GITHUB_TOKEN`。

## `/janitor-precheck`：安装前检查

用于在安装某个新技能前，检查它和已有技能是否重叠。

```text
/janitor-precheck https://github.com/user/repo/tree/main/skills/example
/janitor-precheck ./local-skill
```

底层命令：

```bash
skills-janitor precheck <github-url-or-path>
```

如果用户没有提供 URL 或本地路径，skill 应先询问要检查哪个来源，不能无参数运行。

检查结果分为：

| 结论 | 含义 |
| --- | --- |
| `SAFE` | 未发现显著重叠，可以安装 |
| `MODERATE_OVERLAP` | 有一定重叠，安装前人工确认职责边界 |
| `HIGH_OVERLAP` | 很可能重复，优先考虑使用或改进已有技能 |

## 自然语言触发示例

除了 slash command，用户也可以直接描述意图：

```text
审计我的技能
检查技能有没有重复
我用了哪些 skills
哪些技能浪费 token
安装前检查这个 GitHub skill
搜索 n8n 技能
生成一个 dashboard 给我看
```

助手应将这些请求映射到合适的 janitor skill，并在执行前确认 CLI 存在。

## 推荐组合流程

### 第一次接手一个技能环境

```text
/janitor-audit
/janitor-report
/janitor-tokens --weeks 52
```

目标是先建立全局认知，再处理风险。

### 定期维护

```text
/janitor-report
/janitor-usage --weeks 12
/janitor-fix
```

先报告，再根据 dry-run 输出决定是否应用修复。

### 清理未使用技能

```text
/janitor-usage --weeks 52
/janitor-tokens --weeks 52
/janitor-fix --prune
```

清理时不要只看未使用次数。还要结合 token 成本、重复检测和技能是否仍有业务价值。

### 安装新技能前

```text
/janitor-precheck https://github.com/user/repo/tree/main/skills/example
```

如果结果是中高重叠，先比较已有技能的职责，再决定安装、合并或放弃。

## Legacy fallback

仓库保留 `scripts/*.sh` 作为兼容脚本，但当前技能说明要求正常情况下优先使用 Rust CLI。只有用户明确要求 legacy mode，或需要调试旧路径时，才使用：

```bash
bash scripts/scan.sh
bash scripts/lint.sh
bash scripts/detect_dupes.sh
bash scripts/fix.sh
bash scripts/usage.sh
bash scripts/tokencost.sh
bash scripts/search.sh
bash scripts/precheck.sh
```

Windows 原生环境建议使用 Rust CLI，避免 Bash 行为差异。
