use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::cli::FixArgs;
use crate::frontmatter;
use crate::fs_safety;
use crate::inventory;
use crate::output;
use crate::paths::{self, PlatformPaths};

#[derive(Debug, Clone, Serialize)]
pub struct FixAction {
    pub skill: String,
    pub path: String,
    pub action: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct FixSummary {
    pub fixable_issues: usize,
    pub skipped: usize,
    pub prunable: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FixReport {
    pub dry_run: bool,
    pub prune: bool,
    pub actions: Vec<FixAction>,
    pub skipped: Vec<FixAction>,
    pub summary: FixSummary,
}

pub fn run_fix(args: FixArgs) -> Result<()> {
    let dry_run = args.dry_run || !args.apply;
    let report = build_fix_report(dry_run, args.prune)?;
    if args.json {
        return output::print_json(&report);
    }
    print_fix_report(&report);
    Ok(())
}

pub fn build_fix_report(dry_run: bool, prune: bool) -> Result<FixReport> {
    let paths = PlatformPaths::detect()?;
    let inventory = inventory::build_inventory_with_paths(&paths)?;
    let mut report = FixReport {
        dry_run,
        prune,
        actions: Vec::new(),
        skipped: Vec::new(),
        summary: FixSummary::default(),
    };

    for record in &inventory.skills {
        let path = Path::new(&record.path);
        if record.folder == "skills-janitor" {
            continue;
        }
        if record.is_symlink && record.symlink_target.starts_with("BROKEN:") {
            skip(
                &mut report,
                record.folder.clone(),
                record.path.clone(),
                "Broken symlink - remove manually or use --prune",
            );
            continue;
        }
        let Some(skill_file) = paths::skill_file_in(path) else {
            skip(
                &mut report,
                record.folder.clone(),
                record.path.clone(),
                "No SKILL.md found - create manually",
            );
            continue;
        };
        if fs_safety::is_plugin_or_marketplace_path(path) {
            skip(
                &mut report,
                record.folder.clone(),
                record.path.clone(),
                "Plugin/marketplace skill - don't modify",
            );
            continue;
        }

        let content = fs::read_to_string(&skill_file)?;
        let (new_content, actions) = planned_skill_fixes(&record.folder, &content);
        if !actions.is_empty() {
            report.summary.fixable_issues += actions.len();
            for action in actions {
                report.actions.push(FixAction {
                    skill: record.folder.clone(),
                    path: skill_file.display().to_string(),
                    action,
                    applied: !dry_run,
                });
            }
            if !dry_run {
                fs_safety::atomic_write(&skill_file, &new_content)?;
            }
        }
    }

    if prune {
        for root in paths.skill_roots() {
            for dir in paths::iter_skill_dirs(&root.path)? {
                let name = dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                if name == "skills-janitor" {
                    continue;
                }
                let metadata = fs::symlink_metadata(&dir).ok();
                let is_symlink = metadata
                    .as_ref()
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false);
                if is_symlink && !dir.exists() {
                    let target = fs::read_link(&dir)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| "unknown".to_string());
                    report.summary.prunable += 1;
                    report.actions.push(FixAction {
                        skill: name,
                        path: dir.display().to_string(),
                        action: format!("Broken symlink -> {target} (would remove)"),
                        applied: !dry_run,
                    });
                    if !dry_run {
                        fs_safety::remove_file_checked(&dir, &root.path)?;
                    }
                    continue;
                }
                if dir.is_dir() && paths::skill_file_in(&dir).is_none() {
                    let has_files = walkdir::WalkDir::new(&dir)
                        .min_depth(1)
                        .into_iter()
                        .filter_map(Result::ok)
                        .any(|entry| entry.file_type().is_file());
                    if has_files {
                        skip(
                            &mut report,
                            name,
                            dir.display().to_string(),
                            "No SKILL.md but has files (review manually)",
                        );
                    } else {
                        report.summary.prunable += 1;
                        report.actions.push(FixAction {
                            skill: name,
                            path: dir.display().to_string(),
                            action: "Empty directory (would remove)".to_string(),
                            applied: !dry_run,
                        });
                        if !dry_run {
                            fs_safety::remove_empty_dir_checked(&dir, &root.path)?;
                        }
                    }
                }
            }
        }
    }

    Ok(report)
}

fn skip(report: &mut FixReport, skill: String, path: String, action: impl Into<String>) {
    report.summary.skipped += 1;
    report.skipped.push(FixAction {
        skill,
        path,
        action: action.into(),
        applied: false,
    });
}

pub fn planned_skill_fixes(skill_name: &str, content: &str) -> (String, Vec<String>) {
    let mut new_content = content.to_string();
    let mut actions = Vec::new();
    let first_line = content.lines().next().unwrap_or_default();

    if first_line.trim() != "---"
        && content.lines().take(5).any(|line| {
            line.starts_with("name:")
                || line.starts_with("description:")
                || line.starts_with("version:")
        })
    {
        let lines: Vec<_> = content.lines().collect();
        let mut fm_end = lines.len();
        for (idx, line) in lines.iter().enumerate() {
            if idx == 0 {
                continue;
            }
            let is_fm_line = line.trim().is_empty()
                || line.starts_with("name:")
                || line.starts_with("description:")
                || line.starts_with("version:")
                || line.starts_with("metadata:")
                || line.starts_with(char::is_whitespace);
            if !is_fm_line {
                fm_end = idx;
                break;
            }
        }
        let mut rebuilt = String::from("---\n");
        for line in &lines[..fm_end] {
            rebuilt.push_str(line);
            rebuilt.push('\n');
        }
        rebuilt.push_str("---\n");
        for line in &lines[fm_end..] {
            rebuilt.push_str(line);
            rebuilt.push('\n');
        }
        new_content = rebuilt;
        actions.push("Added missing frontmatter delimiters (---)".to_string());
    }

    let fm = frontmatter::parse_content(&new_content);
    if fm.has_frontmatter && !fm.has_closing_delimiter {
        let mut lines: Vec<String> = new_content.lines().map(ToString::to_string).collect();
        let mut insert_after = 0;
        for (idx, line) in lines.iter().enumerate().skip(1) {
            if is_frontmatter_like(line) {
                insert_after = idx;
            }
        }
        if insert_after > 0 {
            lines.insert(insert_after + 1, "---".to_string());
            new_content = lines.join("\n");
            new_content.push('\n');
            actions.push("Added missing closing --- delimiter".to_string());
        }
    }

    let fm = frontmatter::parse_content(&new_content);
    if fm.has_frontmatter && fm.has_closing_delimiter {
        if frontmatter::extract_field(&fm.raw, "description").is_none() {
            new_content = insert_after_anchor(
                &new_content,
                "name:",
                &format!(
                    "description: \"Use when the user wants to use {skill_name}. Add specific trigger phrases here.\""
                ),
            );
            actions.push("Added template description field".to_string());
        } else if fm.description.trim().is_empty() {
            new_content = replace_line_starting(
                &new_content,
                "description:",
                &format!(
                    "description: \"Use when the user wants to use {skill_name}. Add specific trigger phrases here.\""
                ),
            );
            actions.push("Filled empty description with template".to_string());
        }

        let fm = frontmatter::parse_content(&new_content);
        let has_metadata = fm.raw.lines().any(|line| line.starts_with("metadata:"));
        if fm.version.is_empty() && has_metadata {
            actions.push("metadata: block exists but version missing — add 'version: \"1.0.0\"' under it manually".to_string());
        } else if fm.version.is_empty() {
            if fm.raw.lines().any(|line| line.starts_with("description:")) {
                new_content = insert_after_anchor(
                    &new_content,
                    "description:",
                    "metadata:\n  version: \"1.0.0\"",
                );
            } else {
                new_content =
                    insert_after_anchor(&new_content, "name:", "metadata:\n  version: \"1.0.0\"");
            }
            actions.push("Added missing metadata.version field (1.0.0)".to_string());
        }
    }

    (new_content, actions)
}

fn is_frontmatter_like(line: &str) -> bool {
    line.trim().is_empty()
        || line.starts_with("name:")
        || line.starts_with("description:")
        || line.starts_with("version:")
        || line.starts_with("metadata:")
        || line.starts_with(char::is_whitespace)
}

fn insert_after_anchor(content: &str, anchor: &str, insertion: &str) -> String {
    let mut out = String::new();
    let mut inserted = false;
    for line in content.lines() {
        out.push_str(line);
        out.push('\n');
        if !inserted && line.starts_with(anchor) {
            out.push_str(insertion);
            out.push('\n');
            inserted = true;
        }
    }
    if !inserted {
        let mut lines: Vec<_> = content.lines().collect();
        if lines.first() == Some(&"---") {
            lines.insert(1, insertion);
            let mut rebuilt = lines.join("\n");
            rebuilt.push('\n');
            return rebuilt;
        }
    }
    out
}

fn replace_line_starting(content: &str, prefix: &str, replacement: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        if line.starts_with(prefix) {
            out.push_str(replacement);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn print_fix_report(report: &FixReport) {
    println!("=== Skills Janitor - Auto-Fix ===");
    if report.dry_run {
        println!("Mode: DRY RUN (use --apply to make changes)");
    } else {
        println!("Mode: APPLY (changes written)");
    }
    println!();

    for skipped in &report.skipped {
        println!("  [SKIP]    {}: {}", skipped.skill, skipped.action);
    }
    for action in &report.actions {
        let label = if report.dry_run {
            "DRY RUN"
        } else if action.action.contains("would remove") {
            "PRUNED"
        } else {
            "FIXED"
        };
        println!("  [{label:<7}] {}: {}", action.skill, action.action);
    }
    if report.prune && report.summary.prunable == 0 {
        println!("  No broken or orphaned skills found.");
    }

    println!();
    println!("=== Summary ===");
    println!("  Fixable issues: {}", report.summary.fixable_issues);
    println!("  Skipped:        {}", report.summary.skipped);
    if report.prune {
        println!("  Prunable:       {}", report.summary.prunable);
    }
    if report.dry_run && (report.summary.fixable_issues + report.summary.prunable) > 0 {
        println!();
        println!("  Run with --apply to make these changes.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_metadata_version_without_duplicate() {
        let input = "---\nname: demo\ndescription: Use when testing demo.\n---\nBody\n";
        let (out, actions) = planned_skill_fixes("demo", input);
        assert!(actions.iter().any(|a| a.contains("metadata.version")));
        assert!(out.contains("metadata:\n  version: \"1.0.0\""));
    }

    #[test]
    fn existing_nested_version_is_not_changed() {
        let input = "---\nname: demo\ndescription: Use when testing demo.\nmetadata:\n  version: \"2\"\n---\nBody\n";
        let (out, actions) = planned_skill_fixes("demo", input);
        assert!(actions.is_empty());
        assert_eq!(out, input);
    }
}
