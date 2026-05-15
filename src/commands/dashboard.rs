use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::analysis::dupes;
use crate::analysis::lint;
use crate::analysis::tokens;
use crate::cli::DashboardArgs;
use crate::domain::context::SkillscopeContext;
use crate::domain::paths::PlatformPaths;
use crate::infra::output;

const SNAPSHOT_MARKER_START: &str = "<script type=\"application/json\" id=\"snapshotData\">";
const SNAPSHOT_MARKER_END: &str = "</script>";
const MAX_SNAPSHOTS: usize = 20;
const DASHBOARD_SCHEMA_VERSION: u32 = 2;
const EMBEDDED_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/skillscope-dashboard.html"
));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub schema_version: u32,
    pub timestamp: String,
    pub period_weeks: u32,
    pub budget: u64,
    pub scan: serde_json::Value,
    pub usage: serde_json::Value,
    pub tokens: serde_json::Value,
    pub lint: serde_json::Value,
    pub duplicates: serde_json::Value,
}

pub fn run_dashboard(args: DashboardArgs) -> Result<()> {
    let path = update_dashboard(args.output, args.weeks, args.budget)?;
    if args.open {
        open::that(&path).with_context(|| format!("open {}", path.display()))?;
        println!("Dashboard opened: {}", path.display());
    } else {
        println!("Dashboard saved to: {}", path.display());
    }
    Ok(())
}

pub fn update_dashboard(output: Option<PathBuf>, weeks: u32, budget: u64) -> Result<PathBuf> {
    let paths = PlatformPaths::detect()?;
    update_dashboard_with_paths(&paths, output, weeks, budget)
}

fn update_dashboard_with_paths(
    paths: &PlatformPaths,
    output: Option<PathBuf>,
    weeks: u32,
    budget: u64,
) -> Result<PathBuf> {
    let dashboard = output.unwrap_or_else(|| paths.cwd.join("skillscope-dashboard.html"));
    if let Some(parent) = dashboard.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the embedded template when the file doesn't exist yet, or when an
    // existing file is missing the snapshot marker (legacy/corrupted file).
    let needs_template = if dashboard.is_file() {
        let existing = fs::read_to_string(&dashboard).unwrap_or_default();
        !existing.contains(SNAPSHOT_MARKER_START)
    } else {
        true
    };
    if needs_template {
        fs::write(&dashboard, EMBEDDED_TEMPLATE)?;
    }

    let mut context = SkillscopeContext::with_paths(paths.clone())?;
    let snapshot = build_snapshot_from_context(&mut context, weeks, budget)?;

    let html = fs::read_to_string(&dashboard)?;
    let (new_html, snapshot_count) = append_snapshot_to_html(&html, snapshot)?;
    fs::write(&dashboard, new_html)?;
    println!("Snapshot added ({snapshot_count} total)");
    Ok(dashboard)
}

fn build_snapshot_from_context(
    context: &mut SkillscopeContext,
    weeks: u32,
    budget: u64,
) -> Result<DashboardSnapshot> {
    let inventory = context.inventory();
    let lint = lint::build_lint_report_from_entries(&context.scan.skills);
    let duplicates = dupes::build_duplicate_report_from_records(&inventory.skills);
    let usage = context.usage_report(weeks)?.clone();
    let tokens = tokens::build_token_report_from_scan(&context.scan, Some(&usage), budget);

    Ok(DashboardSnapshot {
        schema_version: DASHBOARD_SCHEMA_VERSION,
        timestamp: Timestamp::now().to_string(),
        period_weeks: weeks,
        budget,
        scan: serde_json::to_value(inventory)?,
        usage: serde_json::to_value(usage)?,
        tokens: serde_json::to_value(tokens)?,
        lint: serde_json::to_value(lint)?,
        duplicates: serde_json::to_value(duplicates)?,
    })
}

fn append_snapshot_to_html(html: &str, snapshot: DashboardSnapshot) -> Result<(String, usize)> {
    let Some(start_idx) = html.find(SNAPSHOT_MARKER_START) else {
        bail!("snapshot marker not found in dashboard");
    };
    let json_start = start_idx + SNAPSHOT_MARKER_START.len();
    let Some(end_rel) = html[json_start..].find(SNAPSHOT_MARKER_END) else {
        bail!("closing script tag not found in dashboard");
    };
    let end_idx = json_start + end_rel;
    let existing_json = html[json_start..end_idx].trim();
    let mut snapshots = if existing_json.is_empty() {
        Vec::<serde_json::Value>::new()
    } else {
        serde_json::from_str::<Vec<serde_json::Value>>(existing_json).unwrap_or_default()
    };
    snapshots.push(serde_json::to_value(snapshot)?);
    if snapshots.len() > MAX_SNAPSHOTS {
        snapshots = snapshots.split_off(snapshots.len() - MAX_SNAPSHOTS);
    }
    let new_json = escape_embedded_json(&serde_json::to_string_pretty(&snapshots)?);
    let count = snapshots.len();
    let new_html = format!("{}{}{}", &html[..json_start], new_json, &html[end_idx..]);
    Ok((new_html, count))
}

fn escape_embedded_json(json: &str) -> String {
    json.replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
}

#[allow(dead_code)]
fn _print_snapshot_json(snapshot: &DashboardSnapshot) -> Result<()> {
    output::print_json(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(id: u32) -> DashboardSnapshot {
        DashboardSnapshot {
            schema_version: DASHBOARD_SCHEMA_VERSION,
            timestamp: format!("2026-05-11T00:00:{id:02}Z"),
            period_weeks: 52,
            budget: 200_000,
            scan: serde_json::json!({ "skills": [] }),
            usage: serde_json::json!({ "active_skills": 0, "unused_skills": 0, "skills": [] }),
            tokens: serde_json::json!({ "total_token_cost": 0, "unused_token_cost": 0, "skills": [] }),
            lint: serde_json::json!({ "summary": { "critical": 0, "warnings": 0, "info": 0, "total": 0 }, "issues": [] }),
            duplicates: serde_json::json!({ "name_collisions": [], "description_overlaps": [] }),
        }
    }

    fn template_with_json(json: &str) -> String {
        format!(
            "<html>{SNAPSHOT_MARKER_START}{json}{SNAPSHOT_MARKER_END}<script>main()</script></html>"
        )
    }

    #[test]
    fn appends_snapshot_at_dashboard_marker() {
        let html = template_with_json("[]");
        let (updated, count) = append_snapshot_to_html(&html, snapshot(1)).unwrap();
        assert_eq!(count, 1);
        assert!(updated.contains("\"schema_version\""));
        assert!(updated.contains("\"usage\""));
        assert!(updated.contains("<script>main()</script>"));
    }

    #[test]
    fn keeps_only_latest_twenty_snapshots() {
        let mut html = template_with_json("[]");
        let mut count = 0;
        for id in 0..25 {
            let result = append_snapshot_to_html(&html, snapshot(id)).unwrap();
            html = result.0;
            count = result.1;
        }
        assert_eq!(count, MAX_SNAPSHOTS);
        let start = html.find(SNAPSHOT_MARKER_START).unwrap() + SNAPSHOT_MARKER_START.len();
        let end = start + html[start..].find(SNAPSHOT_MARKER_END).unwrap();
        let parsed: Vec<DashboardSnapshot> = serde_json::from_str(&html[start..end]).unwrap();
        assert_eq!(parsed.len(), MAX_SNAPSHOTS);
        assert_eq!(parsed.first().unwrap().timestamp, "2026-05-11T00:00:05Z");
    }

    #[test]
    fn escapes_json_for_script_embedding() {
        let mut snap = snapshot(1);
        snap.scan = serde_json::json!({
            "skills": [
                {
                    "folder": "</script><img src=x>",
                    "description": "A & B > C"
                }
            ]
        });
        let (updated, _) = append_snapshot_to_html(&template_with_json("[]"), snap).unwrap();
        let embedded = updated
            .split(SNAPSHOT_MARKER_START)
            .nth(1)
            .unwrap()
            .split(SNAPSHOT_MARKER_END)
            .next()
            .unwrap();
        assert!(!embedded.contains("</script>"));
        assert!(embedded.contains("\\u003c/script\\u003e"));
        assert!(embedded.contains("\\u0026"));
    }

    #[test]
    fn dashboard_snapshot_reuses_existing_context() {
        let home = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        let paths = PlatformPaths::from_home_and_cwd(home.path(), cwd.path());
        let skill = paths.claude_user_skills.join("demo");
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: demo\ndescription: Use when testing dashboard context reuse.\nmetadata:\n  version: \"1.0.0\"\n---\nBody with Gotchas\n",
        )
        .unwrap();

        let mut context = SkillscopeContext::with_paths(paths.clone()).unwrap();
        fs::remove_dir_all(&paths.claude_user_skills).unwrap();
        let snapshot = build_snapshot_from_context(&mut context, 4, 200_000).unwrap();

        assert_eq!(snapshot.scan["skills"].as_array().unwrap().len(), 1);
        assert_eq!(snapshot.tokens["skills"].as_array().unwrap().len(), 1);
        assert_eq!(context.cached_usage_report_count(), 1);
        assert!(!paths.data_dir.join("usage-history.json").exists());
    }
}
