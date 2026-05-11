# 发现与决策

## 需求
- 用户希望把当前分散的 Bash/Python/curl 脚本统一为一个 Rust CLI 应用。
- CLI 需要放在单独文件夹中，计划目录为 `cli/`。
- 安装方式需要支持：
  - `cargo install --git 当前仓库`
  - GitHub Actions 构建并发布二进制。
- 每个 skill 需要把 CLI 检查作为前置步骤：
  - 已安装：运行 `skills-janitor <subcommand>`。
  - 未安装：提示安装 CLI。
- 仓库需要新增 `justfile`，提供 `ci`、`build` 等统一操作。

## 仓库现状发现
- 根目录当前没有 `Cargo.toml`、`Cargo.lock`、Rust 源码、Justfile 或 GitHub workflow。
- `skills/` 下有 7 个 skill：
  - `janitor-audit`
  - `janitor-report`
  - `janitor-fix`
  - `janitor-usage`
  - `janitor-tokens`
  - `janitor-search`
  - `janitor-precheck`
- 7 个 skill 的 `How to Run` 仍然以 `bash <scripts_dir>/*.sh` 为主入口。
- `scripts/` 中有 11 个 shell 脚本：
  - `paths.sh`
  - `scan.sh`
  - `lint.sh`
  - `detect_dupes.sh`
  - `fix.sh`
  - `usage.sh`
  - `tokencost.sh`
  - `search.sh`
  - `compare.sh`
  - `precheck.sh`
  - `dashboard.sh`
- README 当前仍写依赖 `Bash, Python 3, curl`，这与“Windows 原生统一执行层”目标冲突。
- `.claude-plugin/marketplace.json` 当前已有未提交版本号修改：`1.2.0` -> `2.0.0`。实施时需避免误覆盖用户已有改动。

## 技术可行性
- Rust CLI 对该项目高度可行：
  - 文件扫描、frontmatter 解析、JSON 输出、GitHub API、token 估算、usage 历史解析都适合用 Rust 标准库和少量 crate 实现。
  - 运行时可以彻底移除 Python/Bash/curl 依赖。
  - Windows/macOS/Linux 可通过同一源码构建。
- `cargo install --git` 可行：
  - 如果 `Cargo.toml` 位于 `cli/`，安装命令可能需要 `--path` 不适用于 git 子目录；为降低复杂度，需要评估 Cargo 对 git workspace/package 的安装方式。
  - 推荐方案：根目录放 workspace `Cargo.toml`，`cli/` 是 member，且 package/bin 名明确，这样可用 `cargo install --git <repo> --bin skills-janitor`。
  - 如果坚持根目录没有 Cargo workspace，则需要把 crate 放根目录或提供更复杂安装说明。当前用户明确要求 CLI 单独文件夹，因此更适合根 workspace + `cli/` member。
- GitHub 二进制 release 可行：
  - tag 触发 `release.yml`。
  - matrix 构建 Windows/Linux/macOS。
  - 使用 release assets 上传 zip/tar.gz。

## 推荐 Cargo 布局决策

| 决策 | 理由 |
|------|------|
| 根目录增加 workspace `Cargo.toml` | 让 `cargo install --git <repo> --bin skills-janitor` 可发现 workspace 包 |
| `cli/Cargo.toml` 放实际 package | 满足“单独文件夹存放 CLI”要求 |
| package 名建议 `skills-janitor-cli`，bin 名 `skills-janitor` | 避免 package/bin 混淆，同时用户命令短 |
| 根 `justfile` 使用 `--manifest-path cli/Cargo.toml` | 本地任务不依赖当前目录切换 |

## 推荐依赖
- `clap`：CLI 子命令。
- `serde` / `serde_json`：JSON 输出和缓存。
- `serde_yaml` 或自定义轻量 parser：frontmatter。
- `thiserror` / `anyhow`：错误处理。
- `dirs`：用户目录发现。
- `walkdir`：跨平台目录遍历。
- `ureq` 或 `reqwest` + rustls：GitHub API 和 raw 文件下载。
- `time` 或 `chrono`：时间处理。
- `tempfile`：测试和安全写入。

## Justfile 建议

```just
set shell := ["powershell.exe", "-NoProfile", "-Command"]

fmt:
    cargo fmt --manifest-path cli/Cargo.toml

fmt-check:
    cargo fmt --manifest-path cli/Cargo.toml -- --check

clippy:
    cargo clippy --manifest-path cli/Cargo.toml --all-targets --all-features -- -D warnings

test:
    cargo test --manifest-path cli/Cargo.toml --all-targets --all-features

build:
    cargo build --manifest-path cli/Cargo.toml

build-release:
    cargo build --manifest-path cli/Cargo.toml --release

ci: fmt-check clippy test build-release

install-local:
    cargo install --path cli --locked --force
```

注意：上面是 Windows-first 草案。若要让 justfile 在 macOS/Linux 也自然运行，可以不要设置 PowerShell shell，直接使用跨平台简单命令。

## GitHub Actions 建议

### `.github/workflows/ci.yml`
- 触发：
  - `push`
  - `pull_request`
- matrix：
  - `ubuntu-latest`
  - `windows-latest`
  - `macos-latest`
- 步骤：
  - checkout
  - install stable Rust toolchain
  - cache cargo
  - `cargo fmt --manifest-path cli/Cargo.toml -- --check`
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets --all-features -- -D warnings`
  - `cargo test --manifest-path cli/Cargo.toml --all-targets --all-features`
  - `cargo build --manifest-path cli/Cargo.toml --release`

### `.github/workflows/release.yml`
- 触发：
  - tag `v*`
  - 可选 `workflow_dispatch`
- matrix：
  - `x86_64-pc-windows-msvc`
  - `x86_64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
- 产物：
  - Windows zip 包含 `skills-janitor.exe`
  - Unix tar.gz 包含 `skills-janitor`
  - 可选 SHA256 校验文件

## Skill 前置检查建议

每个 `SKILL.md` 增加统一段落：

```markdown
## CLI requirement

Before doing anything else, check whether the Rust CLI is installed:

```bash
skills-janitor --version
```

If it is not installed, stop and tell the user:

```bash
cargo install --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
```

If the user does not have Rust/Cargo, tell them to install from the latest GitHub Release binary instead.
Do not fall back to Python, Bash, or curl scripts unless the user explicitly asks for legacy mode.
```

Windows PowerShell 文档可补：

```powershell
skills-janitor --version
cargo install --git https://github.com/bahayonghang/skills-janitor --bin skills-janitor --locked --force
```

## 主要风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| `cargo install --git` 与 `cli/` 子目录布局冲突 | 安装命令复杂或失败 | 根目录设 Cargo workspace，并验证安装命令 |
| GitHub release 二进制签名/校验缺失 | 用户难以确认下载完整性 | 发布 SHA256 checksums |
| `fix --apply` 写入错误 | 可能破坏用户 skill | 默认 dry-run、原子写入、备份、边界检查 |
| Windows symlink/junction 处理不当 | 误删或误报 | 单独实现 `fs_safety.rs` 并写 Windows 测试 |
| 中文 frontmatter/description 解析回归 | lint/report 数据不准 | fixtures 覆盖中文和多行 YAML |
| 旧用户仍按 Bash 文档操作 | 迁移混乱 | SKILL.md 和 README 同步 CLI-first，脚本标 legacy |

## 资源
- 当前仓库：`D:\Documents\Code\Agents\skills-janitor`
- 当前 skill 目录：`skills/janitor-*`
- 当前脚本目录：`scripts/*.sh`
- 计划新增目录：`cli/`
- 计划新增 CI：`.github/workflows/ci.yml`、`.github/workflows/release.yml`

## 视觉/浏览器发现
<!-- 关键：每执行2次查看/浏览器操作后必须更新此部分 -->
<!-- 多模态内容必须立即以文本形式记录 -->
- 本轮没有浏览器或视觉内容。

---
*每执行2次查看/浏览器/搜索操作后更新此文件*
*防止视觉信息丢失*

## 实施发现：Rust CLI 迁移
- 根目录需要非 virtual package shim，才能让 `cargo install skills-janitor --git <repo> --bin skills-janitor --locked --force` 发现可安装 package；仅根 workspace virtual manifest 会让 `cargo install --path .` 报 `found a virtual manifest`。
- Windows debug 构建扫描真实 Claude history 时很慢；release 构建配合 mtime 过滤、最近文件限制、Aho-Corasick 名称匹配后，`usage --weeks 1 --json` 和 `tokens --weeks 1 --json` 在本机真实历史上约 0.6-0.7s 完成。
- PowerShell 管道 `Select-Object -First` 会提前关闭 stdout，Rust 程序可能出现 `failed printing to stdout: The pipe has been ended` panic；最终烟测改为重定向到临时文件并检查 exit code。
- Windows 上旧的超时 `skills-janitor.exe` 进程会锁住 `target/debug/deps/*.o`，导致后续 test 编译出现 WinError 32；处理方式是先停止遗留进程，再重新运行验证。
- `scripts/*.sh` 已降级为 legacy wrapper，正常路径不再依赖 Bash/Python/curl。
