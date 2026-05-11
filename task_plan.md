# 任务计划：Rust CLI 统一 Skills Janitor 执行层

## 目标
将当前依赖 Bash/Python/curl 的 7 个 Skills Janitor 命令迁移为一个跨平台 Rust CLI，并通过 `cargo install --git`、GitHub Actions 二进制发布、Justfile CI/build/release 入口，以及各 `SKILL.md` 的 CLI 前置检查完成统一交付。

## 当前阶段
阶段 8（完成）

## 范围与成功标准

### 范围内
- 新增独立 CLI 文件夹（建议 `cli/`），内部包含 Rust crate。
- 新增仓库根 `justfile`，提供本地开发与 CI 对齐的命令入口。
- 新增 GitHub Actions workflow：
  - PR/Push CI：fmt、clippy、test、build。
  - Release binary：Windows/macOS/Linux 构建产物上传到 GitHub Release。
- 修改 7 个 `skills/janitor-*/SKILL.md`：
  - 先检查 `skills-janitor --version`。
  - 未安装时提示 `cargo install --git ...` 或下载 GitHub Release 二进制。
  - 运行时统一调用 `skills-janitor <subcommand>`。
- 保留或降级旧 `scripts/*.sh` 为 legacy/fallback，避免一次性破坏 macOS/Linux 旧用户。

### 范围外
- 不在本阶段发布 crates.io 包。
- 不在本阶段删除全部 shell 脚本。
- 不引入 Python/Node 运行时作为 CLI 依赖。
- 不让 skill 自动执行联网安装；只提示用户安装，除非用户明确要求执行安装。

### 成功标准
- Windows PowerShell、macOS、Linux 都能直接运行同一个 CLI 命令族。
- 普通运行不依赖 `python`、`python3`、`bash`、`curl`。
- `cargo install --git https://github.com/bahayonghang/skills-janitor --locked --force` 可安装 CLI。
- GitHub Release 自动产出至少：
  - `skills-janitor-x86_64-pc-windows-msvc.zip`
  - `skills-janitor-x86_64-unknown-linux-gnu.tar.gz`
  - `skills-janitor-x86_64-apple-darwin.tar.gz`
  - 可选：`aarch64-apple-darwin`
- `just ci` 是本地和 GitHub workflow 的主要验证入口。

## 目标结构

```text
skills-janitor/
├── cli/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── src/
│       ├── main.rs
│       ├── cli.rs
│       ├── paths.rs
│       ├── frontmatter.rs
│       ├── inventory.rs
│       ├── lint.rs
│       ├── dupes.rs
│       ├── fix.rs
│       ├── usage.rs
│       ├── tokens.rs
│       ├── github.rs
│       ├── dashboard.rs
│       ├── output.rs
│       └── fs_safety.rs
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── justfile
├── skills/
│   └── janitor-*/SKILL.md
└── scripts/
    └── *.sh  # legacy wrappers or deprecated compatibility scripts
```

## 命令映射

| 当前 skill | 当前脚本 | 新 Rust CLI |
|------------|----------|-------------|
| `janitor-audit` | `scan.sh` | `skills-janitor scan [--json]` |
| `janitor-report` | `scan.sh` + `lint.sh` + `detect_dupes.sh` | `skills-janitor report [--json]` |
| `janitor-fix` | `fix.sh` | `skills-janitor fix [--apply] [--prune]` |
| `janitor-usage` | `usage.sh` | `skills-janitor usage [--weeks N] [--json]` |
| `janitor-tokens` | `tokencost.sh` | `skills-janitor tokens [--budget N] [--weeks N] [--json]` |
| `janitor-search` | `search.sh` + `compare.sh` | `skills-janitor search <keyword>` / `skills-janitor compare <skill-name>` |
| `janitor-precheck` | `precheck.sh` | `skills-janitor precheck <github-url-or-path> [--json]` |
| dashboard helper | `dashboard.sh` | `skills-janitor dashboard [--open] [--output PATH]` |

## 各阶段

### 阶段 1：规划与约束固化
- [x] 确认当前仓库没有 Rust crate、Justfile、GitHub workflow。
- [x] 确认现有 7 个 skill 全部依赖 `bash <scripts_dir>/*.sh`。
- [x] 确认 Rust CLI 迁移方向：新增独立 `cli/` 文件夹。
- [x] 明确 GitHub binary release 和 `cargo install --git` 都是交付目标。
- [x] 创建 `task_plan.md`、`findings.md`、`progress.md`。
- **状态：** complete

### 阶段 2：Rust CLI 脚手架与 Justfile
- [x] 新增根 `Cargo.toml` install shim/workspace 和 `cli/Cargo.toml`，package `skills-janitor-cli`，bin 名称固定为 `skills-janitor`。
- [x] 新增 `src/main.rs` shim、`cli/src/main.rs`、`cli/src/lib.rs` 和 `cli/src/cli.rs`，用 `clap` 定义子命令。
- [x] 新增 `justfile`：`fmt`、`fmt-check`、`clippy`、`test`、`build`、`build-release`/`release-build`、`ci`、`install-local`、`run`。
- [x] 在 README / README_zh 中记录本地安装、开发命令和 CLI 命令族。
- **状态：** complete

### 阶段 3：只读核心能力迁移
- [x] 实现 `paths.rs`：统一发现 Claude/Codex skill 目录，并去重项目目录。
- [x] 实现 `frontmatter.rs`：解析 `SKILL.md` / `Skill.md` 的 frontmatter，覆盖多行 description、`metadata.version`、中文 description。
- [x] 实现 `inventory.rs`：替代 `scan.sh`。
- [x] 实现 `lint.rs`：替代 `lint.sh`。
- [x] 实现 `dupes.rs`：替代 `detect_dupes.sh`。
- [x] 实现 `report` 聚合输出。
- [x] 添加单元测试覆盖 frontmatter、inventory、lint、dupes 等核心行为。
- **状态：** complete

### 阶段 4：分析与联网能力迁移
- [x] 实现 `usage.rs`：替代 `usage.sh`，读取 Claude history/project jsonl，并加入 mtime/数量限制与 Aho-Corasick 匹配优化。
- [x] 实现 `tokens.rs`：替代 `tokencost.sh`，增加中英文 token 粗估并 cross-reference usage。
- [x] 实现 `github.rs`：替代 `search.sh` / `compare.sh` / `precheck.sh` 中的 GitHub API 和 raw fetch。
- [x] 保留 `GITHUB_TOKEN` 环境变量支持。
- [x] 网络错误/rate limit 场景通过 anyhow 上下文输出可读错误。
- **状态：** complete

### 阶段 5：写入与安全操作迁移
- [x] 实现 `fix.rs`：默认 dry-run、`--apply` 才写入、`--prune` 才检查可删除对象、plugin/marketplace/cache 目录只读。
- [x] 实现 `fs_safety.rs`：原子写入、UTF-8 写入、删除前路径边界检查。
- [x] 增加 fix 单元测试覆盖 metadata.version 不重复注入。
- **状态：** complete

### 阶段 6：GitHub Actions 二进制发布
- [x] 新增 `.github/workflows/ci.yml`：Windows/Ubuntu/macOS matrix，fmt、clippy、test、release build。
- [x] 新增 `.github/workflows/release.yml`：tag `v*` / workflow_dispatch 触发，Windows/Linux/macOS/aarch64 macOS matrix，zip/tar.gz 与 SHA256，上传 GitHub Release assets。
- [x] README / README_zh 增加二进制下载安装说明。
- **状态：** complete

### 阶段 7：Skill 文档改造与兼容层
- [x] 更新 7 个 `skills/janitor-*/SKILL.md`：增加 `CLI requirement` 前置检查，未安装时提示 Cargo/GitHub Release，主命令改为 `skills-janitor ...`，legacy Bash 降级为显式 fallback。
- [x] 把旧 `scripts/*.sh` 改成调用 `skills-janitor` 的 legacy wrapper。
- [x] 保留 `.claude-plugin/marketplace.json` 已存在的 2.0.0 修改，未覆盖用户已有改动。
- [x] README / README_zh 同步。
- **状态：** complete

### 阶段 8：端到端验证与交付
- [x] `just ci` 通过。
- [x] `cargo install --path . --locked --force --root target/install-test` 验证根安装 shim 可安装。
- [x] release binary 验证：`skills-janitor --version`、`scan --json`、`report --json`、`tokens --weeks 1 --json`、`usage --weeks 1 --json`、`precheck skills/janitor-audit --json`、`fix --json` 全部 exit=0。
- [x] Windows PowerShell 原生验证，不依赖 WSL/Git Bash/Python/curl。
- [x] 本地等价 release-build 验证：`cargo build --release` 由 `just ci` 执行通过。
- **状态：** complete

## 关键问题
1. CLI 包是否需要同时支持 crates.io 发布？当前计划先只支持 `cargo install --git` 和 GitHub Release binary。
2. Release binary 是否需要覆盖 ARM Linux？当前计划先覆盖 Windows/Linux/macOS 主流目标，可后续扩展。
3. 旧 `scripts/*.sh` 是保留 wrapper、标记 deprecated，还是最终删除？当前计划先保留 wrapper。
4. Skill 安装后是否允许 agent 自动安装 CLI？当前计划只提示安装，避免未经确认执行联网安装。

## 已做决策

| 决策 | 理由 |
|------|------|
| Rust CLI 放在独立 `cli/` 文件夹 | 满足用户要求，同时避免根目录文件与 skill/plugin 资源混杂 |
| bin 名固定为 `skills-janitor` | 方便 skill 前置检查和用户记忆 |
| 根目录新增 `justfile` | 用户要求用 Justfile 统一 ci/build；根目录入口更符合仓库操作习惯 |
| `just ci` 作为本地最终验证入口 | 让本地和 GitHub Actions 的验证链一致 |
| GitHub Release 产出二进制 | 解决没有 Rust 工具链用户的安装门槛 |
| `cargo install --git` 仍保留 | 适合开发者和当前仓库直接安装场景 |
| 旧 shell 脚本先保留 | 降低迁移风险，避免一次性破坏已有用户 |

## 遇到的错误

| 错误 | 尝试次数 | 解决方案 |
|------|---------|---------|
| 当前仓库没有现成 Rust/CI 结构 | 1 | 计划从独立 `cli/` crate、根 `justfile`、`.github/workflows` 开始 |
| 当前 skill 入口写死 Bash | 1 | 计划在 7 个 `SKILL.md` 中增加 CLI 前置检查并改为 `skills-janitor` 命令 |

## 备注
- 重大实现前重新读取 `task_plan.md`、`findings.md`、`progress.md`。
- 外部 GitHub workflow/release 文档如需使用最新 action 版本，实施前应再以官方文档确认。
- 本计划只创建实施计划；尚未开始代码实现。

## 实施完成摘要（2026-05-11）
- 新增 Rust CLI：根 install shim + `cli/` 实际 crate。
- 新增 CI/release workflow 与根 `justfile`。
- 7 个 skill 文档改为 CLI-first，旧脚本变为 legacy wrapper。
- README/README_zh 同步 Rust CLI 安装、开发、命令与二进制 release 说明。
- 验证：`just ci` 通过；release CLI 全命令烟测通过。
