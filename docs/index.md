# Skills Janitor 文档

Skills Janitor 是一个用于审计、追踪和维护 AI 编程技能生态的工具集。它同时提供跨平台 Rust CLI 和一组可被 Claude Code / OpenAI Codex 调用的技能命令，用于发现损坏技能、重复技能、未使用技能和上下文 token 浪费。

## 适用场景

- 你安装了很多 `SKILL.md`，想知道哪些还在被使用。
- 你想在安装新技能前检查它是否和已有技能重复。
- 你想审计 Claude Code 与 Codex 的用户级、项目级技能目录。
- 你想用 JSON 输出接入脚本、CI 或其他自动化流程。
- 你想生成本地 HTML dashboard，快速查看技能健康状态。

## 两种使用入口

### Rust CLI

CLI 名为 `skills-janitor`，是当前推荐的底层执行入口。它不依赖 Python、Bash、curl 或 Node 运行时，适合本地终端、CI、脚本和跨平台使用。

```bash
skills-janitor report
skills-janitor usage --weeks 12
skills-janitor dashboard --open --weeks 52
```

继续阅读：[CLI 使用方法](/cli)。

### Skills 命令

仓库内置 7 个 `janitor-*` 技能。它们面向 AI 助手对话体验，用户可以通过 slash command 或自然语言触发。每个 skill 会先调用 Rust CLI，因此正常路径仍然由同一套 CLI 实现负责。

```text
/janitor-audit
/janitor-report
/janitor-fix --prune
```

继续阅读：[Skills 使用方法](/skills)。

## 安装 CLI

使用 Cargo 从仓库安装：

```bash
cargo install skills-janitor --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
```

本地开发时可在仓库根目录运行：

```bash
just ci
just install
skills-janitor --version
```

`just install` 还会把仓库内置 skills 同步到当前项目 `.claude/skills/` 和 `.agents/skills/`；需要额外测试 Kiro 等目标时可运行 `just install --target kiro`。

如果没有 Rust 工具链，可以从 GitHub Releases 下载 Windows、Linux 或 macOS 预编译二进制。

## 文档站开发

本目录是独立 VitePress 文档站：

```bash
cd docs
npm install
npm run dev
```

生产构建：

```bash
cd docs
npm run build
```
