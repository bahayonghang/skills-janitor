use anyhow::Result;
use serde::Serialize;

use crate::dupes::{self, DuplicateReport};
use crate::frontmatter;
use crate::inventory::{self, SkillRecord};
use crate::output;

#[derive(Debug, Clone, Serialize)]
pub struct LintIssue {
    pub severity: Severity,
    pub skill: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LintSummary {
    pub critical: usize,
    pub warnings: usize,
    pub info: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LintReport {
    pub summary: LintSummary,
    pub issues: Vec<LintIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub lint: LintReport,
    pub duplicates: DuplicateReport,
}

pub fn build_report() -> Result<HealthReport> {
    let inventory = inventory::build_inventory()?;
    let lint = build_lint_report_from_records(&inventory.skills);
    let duplicates = dupes::build_duplicate_report_from_records(&inventory.skills);
    Ok(HealthReport { lint, duplicates })
}

pub fn print_report() -> Result<()> {
    let report = build_report()?;
    print_lint_report(&report.lint);
    println!();
    dupes::print_duplicate_report(&report.duplicates);
    Ok(())
}

pub fn build_lint_report() -> Result<LintReport> {
    Ok(build_lint_report_from_records(
        &inventory::all_skill_records()?,
    ))
}

pub fn build_lint_report_from_records(records: &[SkillRecord]) -> LintReport {
    let mut issues = Vec::new();
    for record in records {
        lint_record(record, &mut issues);
    }
    let mut summary = LintSummary::default();
    for issue in &issues {
        match issue.severity {
            Severity::Critical => summary.critical += 1,
            Severity::Warning => summary.warnings += 1,
            Severity::Info => summary.info += 1,
        }
    }
    summary.total = summary.critical + summary.warnings + summary.info;
    LintReport { summary, issues }
}

fn lint_record(record: &SkillRecord, issues: &mut Vec<LintIssue>) {
    if record.folder == "skills-janitor" {
        return;
    }
    if record.is_symlink && record.symlink_target.starts_with("BROKEN:") {
        push(
            issues,
            Severity::Critical,
            record,
            format!(
                "Broken symlink -> {}",
                record.symlink_target.trim_start_matches("BROKEN:")
            ),
        );
        return;
    }
    if !record.has_skill_file {
        push(
            issues,
            Severity::Critical,
            record,
            "Missing SKILL.md - directory is not a loadable skill",
        );
        return;
    }
    if !record.has_frontmatter {
        push(
            issues,
            Severity::Critical,
            record,
            "Missing frontmatter (no opening ---)",
        );
        return;
    }

    let skill_file = crate::paths::skill_file_in(std::path::Path::new(&record.path));
    let fm = if let Some(skill_file) = &skill_file {
        frontmatter::parse_file(skill_file).unwrap_or_default()
    } else {
        frontmatter::Frontmatter {
            has_frontmatter: record.has_frontmatter,
            has_closing_delimiter: record.has_frontmatter,
            name: record.name.clone(),
            description: record.description.clone(),
            version: record.version.clone(),
            line_count: record.line_count,
            ..Default::default()
        }
    };
    if !fm.has_closing_delimiter {
        push(
            issues,
            Severity::Critical,
            record,
            "Missing closing --- in frontmatter",
        );
        return;
    }

    if record.name.trim().is_empty() {
        push(
            issues,
            Severity::Warning,
            record,
            "Missing 'name' field in frontmatter",
        );
    } else if !name_matches(&record.folder, &record.name) {
        push(
            issues,
            Severity::Info,
            record,
            format!(
                "Folder name '{}' doesn't match skill name '{}'",
                record.folder, record.name
            ),
        );
    }

    let desc = record.description.trim();
    if desc.is_empty() {
        push(
            issues,
            Severity::Critical,
            record,
            "Missing 'description' field - Claude can't trigger this skill",
        );
    } else {
        let desc_len = desc.chars().count();
        if desc_len < 30 {
            push(
                issues,
                Severity::Warning,
                record,
                format!(
                    "Description too short ({desc_len} chars) - should be 50-200 for good triggering"
                ),
            );
        } else if desc_len > 500 {
            push(
                issues,
                Severity::Info,
                record,
                format!("Description is long ({desc_len} chars) - consider trimming to < 300"),
            );
        }
        if !explains_trigger(desc) {
            push(
                issues,
                Severity::Warning,
                record,
                "Description doesn't explain when to trigger - add 'Use when...' or 'Also use when...'",
            );
        }
        if fm.disable_model_invocation == Some(true) && desc_len < 50 {
            push(
                issues,
                Severity::Warning,
                record,
                "Skill uses disable-model-invocation but description is very short — Claude may not trigger it correctly",
            );
        }
    }

    if fm.body_non_empty_lines() < 3 {
        push(
            issues,
            Severity::Warning,
            record,
            format!(
                "Very little body content ({} non-empty lines)",
                fm.body_non_empty_lines()
            ),
        );
    }
    if let Some(skill_file) = &skill_file {
        let content = std::fs::read_to_string(skill_file).unwrap_or_default();
        if !content.to_lowercase().contains("gotcha") {
            push(
                issues,
                Severity::Info,
                record,
                "No Gotchas section - consider adding common pitfalls",
            );
        }
    }
    if record.line_count > 500 {
        push(
            issues,
            Severity::Info,
            record,
            format!(
                "Skill file is large ({} lines) - consider using progressive disclosure with reference files",
                record.line_count
            ),
        );
    }
}

fn name_matches(folder: &str, name: &str) -> bool {
    let folder = folder.to_lowercase();
    let normalized_name = name.to_lowercase().replace(' ', "-");
    folder == name.to_lowercase() || folder == normalized_name
}

fn explains_trigger(desc: &str) -> bool {
    let desc = desc.to_lowercase();
    [
        "when",
        "trigger",
        "use for",
        "invoke",
        "mention",
        "says",
        "asks",
        "also use",
        "use this",
        "relevant",
        "appropriate",
        "helps with",
        "designed for",
    ]
    .iter()
    .any(|needle| desc.contains(needle))
}

fn push(
    issues: &mut Vec<LintIssue>,
    severity: Severity,
    record: &SkillRecord,
    message: impl Into<String>,
) {
    issues.push(LintIssue {
        severity,
        skill: record.folder.clone(),
        message: message.into(),
        path: record.path.clone(),
    });
}

pub fn print_lint_report(report: &LintReport) {
    println!("=== Skills Janitor - Lint Report ===");
    println!();
    for issue in &report.issues {
        let label = match issue.severity {
            Severity::Critical => "CRITICAL",
            Severity::Warning => "WARNING ",
            Severity::Info => "INFO    ",
        };
        println!("  [{label}] {}: {}", issue.skill, issue.message);
    }
    println!();
    println!("=== Summary ===");
    println!("  Critical: {}", report.summary.critical);
    println!("  Warnings: {}", report.summary.warnings);
    println!("  Info:     {}", report.summary.info);
    println!("  Total:    {}", report.summary.total);
}

#[allow(dead_code)]
pub fn run_lint_json() -> Result<()> {
    output::print_json(&build_lint_report()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_missing_description() {
        let record = SkillRecord {
            folder: "a".into(),
            scope: "user".into(),
            platform: "claude".into(),
            path: "missing".into(),
            real_path: "missing".into(),
            is_symlink: false,
            symlink_target: String::new(),
            has_skill_file: true,
            name: "a".into(),
            description: String::new(),
            version: String::new(),
            has_frontmatter: true,
            has_body: false,
            line_count: 4,
            extra_files: 0,
        };
        let report = build_lint_report_from_records(&[record]);
        assert!(report.summary.critical >= 1);
    }

    #[test]
    fn flags_missing_skill_file_as_critical() {
        let record = SkillRecord {
            folder: "learned".into(),
            scope: "user".into(),
            platform: "claude".into(),
            path: "learned".into(),
            real_path: "learned".into(),
            is_symlink: false,
            symlink_target: String::new(),
            has_skill_file: false,
            name: String::new(),
            description: String::new(),
            version: String::new(),
            has_frontmatter: false,
            has_body: false,
            line_count: 0,
            extra_files: 2,
        };
        let report = build_lint_report_from_records(&[record]);
        assert_eq!(report.summary.critical, 1);
        assert_eq!(report.issues[0].skill, "learned");
        assert!(report.issues[0].message.contains("Missing SKILL.md"));
    }
}
