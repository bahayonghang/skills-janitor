use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgAction, Args, Parser, Subcommand};

use crate::analysis::{lint, tokens, usage};
use crate::commands::{dashboard, fix, github, skill_install};
use crate::domain::inventory;

#[derive(Debug, Parser)]
#[command(name = "skillscope")]
#[command(author, version, about = "Local skill audit and dashboard for Claude Code and Codex skills", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Scan installed skills and emit an inventory.
    Scan(JsonArgs),
    /// Run lint and duplicate checks together.
    Report(JsonArgs),
    /// Fix common skill metadata problems. Dry-run by default.
    Fix(FixArgs),
    /// Analyze which skills appear in recent local conversation history.
    Usage(WeeksJsonArgs),
    /// Estimate context-window token cost per installed skill.
    Tokens(TokenArgs),
    /// Search GitHub repositories for matching skills.
    Search(SearchArgs),
    /// Compare one installed skill against GitHub alternatives.
    Compare(CompareArgs),
    /// Check a local path or GitHub URL before installing a new skill.
    Precheck(PrecheckArgs),
    /// Update and optionally open the HTML dashboard.
    Dashboard(DashboardArgs),
    /// Install bundled repository skills into project skill roots.
    #[command(hide = true)]
    InstallSkills(InstallSkillsArgs),
}

#[derive(Debug, Args, Clone, Copy)]
pub struct JsonArgs {
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, Copy)]
pub struct WeeksJsonArgs {
    /// Number of weeks of history to inspect.
    #[arg(long, default_value_t = 4)]
    pub weeks: u32,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, Copy)]
pub struct TokenArgs {
    /// Context window budget used for percentage calculations.
    #[arg(long, default_value_t = 200_000)]
    pub budget: u64,
    /// Number of weeks of usage data to cross-reference.
    #[arg(long, default_value_t = 4)]
    pub weeks: u32,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct FixArgs {
    /// Actually write changes. Without this flag, fix runs as a dry-run preview.
    #[arg(long)]
    pub apply: bool,
    /// Force dry-run mode even if other flags are present.
    #[arg(long)]
    pub dry_run: bool,
    /// Also find broken symlinks and empty skill directories.
    #[arg(long)]
    pub prune: bool,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SearchArgs {
    /// Keyword to search for on GitHub.
    pub keyword: String,
    /// Maximum number of search results.
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct CompareArgs {
    /// Name of an installed skill to compare.
    pub skill_name: String,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct PrecheckArgs {
    /// GitHub URL or local path for the skill to check.
    pub source: String,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct DashboardArgs {
    /// Open the dashboard in the default browser after updating it.
    #[arg(long)]
    pub open: bool,
    /// Output path for the generated dashboard HTML.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Number of weeks of history to inspect for usage and token activity.
    #[arg(long, default_value_t = 52)]
    pub weeks: u32,
    /// Context window budget used for token percentage calculations.
    #[arg(long, default_value_t = 200_000)]
    pub budget: u64,
}

#[derive(Debug, Args, Clone)]
pub struct InstallSkillsArgs {
    /// Extra project target to install into, in addition to claude and agents.
    ///
    /// Built-in targets include `claude`, `agents`/`codex`, and `kiro`.
    /// Unknown names resolve to `.<name>/skills` under the project directory.
    #[arg(long, value_name = "TARGET", action = ArgAction::Append)]
    pub target: Vec<String>,
    /// Project directory used to resolve target roots.
    #[arg(long, hide = true)]
    pub project: Option<PathBuf>,
    /// Source directory containing skill folders.
    #[arg(long, hide = true)]
    pub source: Option<PathBuf>,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan(args) => inventory::run_scan(args.json),
        Commands::Report(args) => lint::run_report(args.json),
        Commands::Fix(args) => fix::run_fix(args),
        Commands::Usage(args) => usage::run_usage(args.weeks, args.json),
        Commands::Tokens(args) => tokens::run_tokens(args.budget, args.weeks, args.json),
        Commands::Search(args) => github::run_search(args),
        Commands::Compare(args) => github::run_compare(args),
        Commands::Precheck(args) => github::run_precheck(args),
        Commands::Dashboard(args) => dashboard::run_dashboard(args),
        Commands::InstallSkills(args) => skill_install::run_install_skills(args),
    }
}
