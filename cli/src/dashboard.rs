use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::cli::DashboardArgs;
use crate::inventory;
use crate::lint;
use crate::output;
use crate::paths::PlatformPaths;
use crate::{tokens, usage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub timestamp: String,
    pub scan: serde_json::Value,
    pub tokens: serde_json::Value,
    pub usage: serde_json::Value,
    pub lint: serde_json::Value,
}

pub fn run_dashboard(args: DashboardArgs) -> Result<()> {
    let path = update_dashboard(args.output)?;
    if args.open {
        open::that(&path).with_context(|| format!("open {}", path.display()))?;
        println!("Dashboard opened: {}", path.display());
    } else {
        println!("Dashboard saved to: {}", path.display());
    }
    Ok(())
}

pub fn update_dashboard(output: Option<PathBuf>) -> Result<PathBuf> {
    let paths = PlatformPaths::detect()?;
    let dashboard = output.unwrap_or_else(|| paths.cwd.join("data").join("janitor-dashboard.html"));
    if let Some(parent) = dashboard.parent() {
        fs::create_dir_all(parent)?;
    }
    if !dashboard.is_file() {
        let template = paths.cwd.join("templates").join("janitor-dashboard.html");
        if !template.is_file() {
            bail!("Dashboard template not found: {}", template.display());
        }
        fs::copy(&template, &dashboard)?;
    }

    let snapshot = DashboardSnapshot {
        timestamp: Timestamp::now().to_string(),
        scan: serde_json::to_value(inventory::build_inventory()?)?,
        tokens: serde_json::to_value(tokens::build_token_report(200_000, 52)?)?,
        usage: serde_json::to_value(usage::build_usage_report(52)?)?,
        lint: serde_json::to_value(lint::build_lint_report()?)?,
    };

    let html = fs::read_to_string(&dashboard)?;
    let marker_start = "<script type=\"application/json\" id=\"snapshotData\">";
    let marker_end = "</script>";
    let Some(start_idx) = html.find(marker_start) else {
        bail!("snapshot marker not found in dashboard");
    };
    let json_start = start_idx + marker_start.len();
    let Some(end_rel) = html[json_start..].find(marker_end) else {
        bail!("closing script tag not found in dashboard");
    };
    let end_idx = json_start + end_rel;
    let existing_json = html[json_start..end_idx].trim();
    let mut snapshots = if existing_json.is_empty() {
        Vec::<DashboardSnapshot>::new()
    } else {
        serde_json::from_str::<Vec<DashboardSnapshot>>(existing_json).unwrap_or_default()
    };
    snapshots.push(snapshot);
    if snapshots.len() > 20 {
        snapshots = snapshots.split_off(snapshots.len() - 20);
    }
    let new_json = serde_json::to_string_pretty(&snapshots)?;
    let new_html = format!("{}{}{}", &html[..json_start], new_json, &html[end_idx..]);
    fs::write(&dashboard, new_html)?;
    println!("Snapshot added ({} total)", snapshots.len());
    Ok(dashboard)
}

#[allow(dead_code)]
fn _print_snapshot_json(snapshot: &DashboardSnapshot) -> Result<()> {
    output::print_json(snapshot)
}
