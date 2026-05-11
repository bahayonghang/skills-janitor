use anyhow::Result;
use serde::Serialize;

use crate::inventory::{self, SkillRecord};
use crate::output;
use crate::usage;

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
    let inventory = inventory::build_inventory()?;
    let usage = usage::build_usage_report(weeks).ok();
    let mut used_names = std::collections::HashSet::new();
    if let Some(usage) = usage {
        for skill in usage.skills.into_iter().filter(|s| s.count > 0) {
            used_names.insert(skill.name);
        }
    }

    let mut records = inventory.skills;
    records.sort_by(|a, b| a.real_path.cmp(&b.real_path));
    records.dedup_by(|a, b| a.real_path == b.real_path && !a.real_path.is_empty());

    let mut skills = records
        .iter()
        .filter(|record| record.has_skill_file && record.folder != "skills-janitor")
        .map(|record| token_cost(record, budget, used_names.contains(&record.folder)))
        .collect::<Vec<_>>();
    skills.sort_by(|a, b| b.tokens.cmp(&a.tokens).then(a.name.cmp(&b.name)));
    let total_token_cost: u64 = skills.iter().map(|s| s.tokens).sum();
    let unused_token_cost: u64 = skills.iter().filter(|s| !s.used).map(|s| s.tokens).sum();
    Ok(TokenReport {
        budget,
        total_token_cost,
        total_budget_percent: percent(total_token_cost, budget),
        unused_token_cost,
        unused_budget_percent: percent(unused_token_cost, budget),
        skills,
    })
}

fn token_cost(record: &SkillRecord, budget: u64, used: bool) -> SkillTokenCost {
    let chars = std::fs::read_to_string(
        crate::paths::skill_file_in(std::path::Path::new(&record.path)).unwrap_or_default(),
    )
    .map(|s| s.chars().count() as u64)
    .unwrap_or_else(|_| {
        record.description.chars().count() as u64 + (record.line_count as u64 * 24)
    });
    let tokens = estimate_tokens(chars, &record.description);
    SkillTokenCost {
        name: record.folder.clone(),
        scope: record.scope.clone(),
        tokens,
        budget_percent: percent(tokens, budget),
        used,
        path: record.path.clone(),
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
    println!("=== Skills Janitor - Context Window Cost ===");
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
