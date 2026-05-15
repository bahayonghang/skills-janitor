use anyhow::Result;
use serde::Serialize;

use crate::analysis::usage;
use crate::analysis::usage::UsageReport;
use crate::domain::inventory::{self, ScanSnapshot, SkillEntry};
use crate::domain::paths::PlatformPaths;
use crate::infra::output;

#[derive(Debug, Clone, Serialize)]
pub struct SkillTokenCost {
    pub name: String,
    pub scope: String,
    pub tokens: u64,
    pub budget_percent: f64,
    pub used: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenReport {
    pub budget: u64,
    pub total_token_cost: u64,
    pub total_budget_percent: f64,
    pub unused_token_cost: u64,
    pub unused_budget_percent: f64,
    pub skills: Vec<SkillTokenCost>,
}

pub fn run_tokens(budget: u64, weeks: u32, json: bool) -> Result<()> {
    let report = build_token_report(budget, weeks)?;
    if json {
        output::print_json(&report)
    } else {
        print_token_report(&report);
        Ok(())
    }
}

pub fn build_token_report(budget: u64, weeks: u32) -> Result<TokenReport> {
    let paths = PlatformPaths::detect()?;
    build_token_report_with_paths(&paths, budget, weeks)
}

pub fn build_token_report_with_paths(
    paths: &PlatformPaths,
    budget: u64,
    weeks: u32,
) -> Result<TokenReport> {
    let scan = inventory::scan_snapshot_with_paths(paths)?;
    let usage = usage::build_usage_report_from_snapshot(paths, &scan, weeks).ok();
    Ok(build_token_report_from_scan(&scan, usage.as_ref(), budget))
}

pub fn build_token_report_from_scan(
    scan: &ScanSnapshot,
    usage: Option<&UsageReport>,
    budget: u64,
) -> TokenReport {
    let mut used_names = std::collections::HashSet::new();
    if let Some(usage) = usage {
        for skill in usage.skills.iter().filter(|s| s.count > 0) {
            used_names.insert(skill.name.clone());
        }
    }

    let mut entries = scan.skills.iter().collect::<Vec<_>>();
    entries.sort_by(|a, b| a.record.real_path.cmp(&b.record.real_path));
    entries.dedup_by(|a, b| {
        a.record.real_path == b.record.real_path && !a.record.real_path.is_empty()
    });

    let mut skills = entries
        .iter()
        .filter(|entry| entry.record.has_skill_file && entry.record.folder != "skillscope")
        .map(|entry| token_cost(entry, budget, used_names.contains(&entry.record.folder)))
        .collect::<Vec<_>>();
    skills.sort_by(|a, b| b.tokens.cmp(&a.tokens).then(a.name.cmp(&b.name)));
    let total_token_cost: u64 = skills.iter().map(|s| s.tokens).sum();
    let unused_token_cost: u64 = skills.iter().filter(|s| !s.used).map(|s| s.tokens).sum();
    TokenReport {
        budget,
        total_token_cost,
        total_budget_percent: percent(total_token_cost, budget),
        unused_token_cost,
        unused_budget_percent: percent(unused_token_cost, budget),
        skills,
    }
}

fn token_cost(entry: &SkillEntry, budget: u64, used: bool) -> SkillTokenCost {
    let record = &entry.record;
    let chars = entry.skill_file_chars.unwrap_or_else(|| {
        record.description.chars().count() as u64 + (record.line_count as u64 * 24)
    });
    let tokens = estimate_tokens(chars, &record.description);
    SkillTokenCost {
        name: record.folder.clone(),
        scope: record.scope.clone(),
        tokens,
        budget_percent: percent(tokens, budget),
        used,
        path: entry.dir.display().to_string(),
    }
}

fn estimate_tokens(chars: u64, text: &str) -> u64 {
    let cjk = text
        .chars()
        .filter(|ch| ('\u{4e00}'..='\u{9fff}').contains(ch))
        .count() as u64;
    let ascii_chars = chars.saturating_sub(cjk);
    ((ascii_chars as f64 / 4.0).ceil() as u64) + cjk
}

fn percent(n: u64, d: u64) -> f64 {
    if d == 0 {
        0.0
    } else {
        ((n as f64 / d as f64) * 1000.0).round() / 10.0
    }
}

fn print_token_report(report: &TokenReport) {
    println!("=== Skillscope - Context Window Cost ===");
    println!("Budget: {} tokens", report.budget);
    println!();
    println!("  {:<35} {:>8} {:>8} Used?", "Skill", "Tokens", "Budget");
    for skill in report.skills.iter().take(40) {
        println!(
            "  {:<35} {:>8} {:>7.1}% {}",
            skill.name,
            skill.tokens,
            skill.budget_percent,
            if skill.used { "yes" } else { "NO" }
        );
    }
    println!();
    println!(
        "  Total token cost: {} ({:.1}% of budget)",
        report.total_token_cost, report.total_budget_percent
    );
    println!(
        "  Unused skill cost: {} ({:.1}% of budget wasted)",
        report.unused_token_cost, report.unused_budget_percent
    );
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn token_builder_does_not_persist_usage_history() {
        let home = tempdir().unwrap();
        let cwd = tempdir().unwrap();
        let paths = PlatformPaths::from_home_and_cwd(home.path(), cwd.path());
        let skill = paths.claude_user_skills.join("demo");
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: demo\ndescription: Use when testing demo token estimates.\nmetadata:\n  version: \"1.0.0\"\n---\nBody\n",
        )
        .unwrap();

        let report = build_token_report_with_paths(&paths, 200_000, 4).unwrap();

        assert_eq!(report.skills.len(), 1);
        assert!(!paths.data_dir.join("usage-history.json").exists());
    }
}
