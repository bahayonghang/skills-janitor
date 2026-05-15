use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use anyhow::Result;
use jiff::{Timestamp, ToSpan};
use serde::{Deserialize, Serialize};

use crate::analysis::dupes;
use crate::domain::inventory::{self, ScanSnapshot};
use crate::domain::paths::PlatformPaths;
use crate::infra::output;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUsage {
    pub name: String,
    pub scope: String,
    pub count: usize,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReport {
    pub period_weeks: u32,
    pub active_skills: usize,
    pub total_skills: usize,
    pub unused_skills: usize,
    pub most_used: Option<SkillUsage>,
    pub skills: Vec<SkillUsage>,
}

pub fn run_usage(weeks: u32, json: bool) -> Result<()> {
    let paths = PlatformPaths::detect()?;
    let scan = inventory::scan_snapshot_with_paths(&paths)?;
    let report = build_usage_report_from_snapshot(&paths, &scan, weeks)?;
    persist_usage_history(&paths, &report).ok();
    if json {
        output::print_json(&report)
    } else {
        print_usage_report(&report);
        Ok(())
    }
}

#[allow(dead_code)]
pub fn build_usage_report(weeks: u32) -> Result<UsageReport> {
    let paths = PlatformPaths::detect()?;
    let scan = inventory::scan_snapshot_with_paths(&paths)?;
    build_usage_report_from_snapshot(&paths, &scan, weeks)
}

pub fn build_usage_report_from_snapshot(
    paths: &PlatformPaths,
    scan: &ScanSnapshot,
    weeks: u32,
) -> Result<UsageReport> {
    let installed = dupes::installed_keyword_map_from_entries(&scan.skills);
    let counts = collect_usage_counts(paths, weeks, &installed)?;
    let mut skills = installed
        .into_iter()
        .map(|(name, scope, _keywords, description)| SkillUsage {
            count: *counts.get(&name).unwrap_or(&0),
            name,
            scope,
            description,
        })
        .collect::<Vec<_>>();
    skills.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
    let active_skills = skills.iter().filter(|s| s.count > 0).count();
    let total_skills = skills.len();
    let unused_skills = total_skills.saturating_sub(active_skills);
    let most_used = skills.first().filter(|s| s.count > 0).cloned();
    let report = UsageReport {
        period_weeks: weeks,
        active_skills,
        total_skills,
        unused_skills,
        most_used,
        skills,
    };
    Ok(report)
}

fn collect_usage_counts(
    paths: &PlatformPaths,
    weeks: u32,
    installed: &[(String, String, HashSet<String>, String)],
) -> Result<HashMap<String, usize>> {
    let matcher = UsageMatcher::new(installed);
    let mut counts = HashMap::new();
    let cutoff = Timestamp::now()
        .checked_sub((weeks.max(1) as i64 * 7).days())
        .unwrap_or(Timestamp::UNIX_EPOCH);
    let cutoff_system = if weeks >= 52 {
        std::time::UNIX_EPOCH
    } else {
        timestamp_to_system_time(cutoff)
    };

    for history in paths
        .history_candidates()
        .into_iter()
        .filter(|path| path.is_file())
    {
        if modified_after(&history, cutoff_system) {
            scan_jsonl_file(&history, cutoff, &matcher, &mut counts)?;
        }
    }
    for file in recent_conversation_files(paths, cutoff_system, 120)? {
        scan_jsonl_file(&file, cutoff, &matcher, &mut counts)?;
    }
    Ok(counts)
}

fn recent_conversation_files(
    paths: &PlatformPaths,
    cutoff: std::time::SystemTime,
    limit: usize,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for dir in paths.conversation_dirs() {
        collect_jsonl_files(&dir, cutoff, &mut files)?;
    }
    files.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));
    files.truncate(limit);
    Ok(files.into_iter().map(|(path, _)| path).collect())
}

fn collect_jsonl_files(
    dir: &Path,
    cutoff: std::time::SystemTime,
    files: &mut Vec<(PathBuf, std::time::SystemTime)>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file()
                && path.extension().and_then(|e| e.to_str()) == Some("jsonl")
            {
                let modified = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                if modified >= cutoff {
                    files.push((path, modified));
                }
            }
        }
    }
    Ok(())
}

fn scan_jsonl_file(
    path: &std::path::Path,
    cutoff: Timestamp,
    matcher: &UsageMatcher,
    counts: &mut HashMap<String, usize>,
) -> Result<()> {
    let file = fs::File::open(path)?;
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if !(line.contains("skill")
            || line.contains("Skill")
            || line.contains("/janitor")
            || line.contains("<skill>")
            || line.contains("\"name\":\""))
        {
            continue;
        }
        let names = matcher.names_in(&line);
        if names.is_empty() {
            continue;
        }
        let recent_enough = extract_timestamp(&line)
            .map(|ts| ts >= cutoff)
            .unwrap_or(true);
        if !recent_enough {
            continue;
        }
        for name in names {
            *counts.entry(name).or_insert(0) += 1;
        }
    }
    Ok(())
}

struct UsageMatcher {
    automaton: Option<AhoCorasick>,
    pattern_to_name: Vec<String>,
}

impl UsageMatcher {
    fn new(installed: &[(String, String, HashSet<String>, String)]) -> Self {
        let mut patterns = Vec::new();
        let mut pattern_to_name = Vec::new();
        for (name, _scope, _keywords, _description) in installed {
            for pattern in [format!("/{name}"), format!("\"{name}\"")] {
                patterns.push(pattern);
                pattern_to_name.push(name.clone());
            }
        }
        let automaton = if patterns.is_empty() {
            None
        } else {
            AhoCorasickBuilder::new()
                .ascii_case_insensitive(true)
                .build(patterns)
                .ok()
        };
        Self {
            automaton,
            pattern_to_name,
        }
    }

    fn names_in(&self, line: &str) -> Vec<String> {
        let Some(automaton) = &self.automaton else {
            return Vec::new();
        };
        let mut names = HashSet::new();
        for mat in automaton.find_iter(line) {
            if let Some(name) = self.pattern_to_name.get(mat.pattern().as_usize()) {
                names.insert(name.clone());
            }
        }
        names.into_iter().collect()
    }
}

fn extract_timestamp(line: &str) -> Option<Timestamp> {
    for key in ["\"timestamp\":\"", "\"created_at\":\"", "\"date\":\""] {
        if let Some(start) = line.find(key) {
            let value_start = start + key.len();
            let rest = &line[value_start..];
            let Some(end) = rest.find('"') else {
                continue;
            };
            if let Ok(ts) = rest[..end].parse::<Timestamp>() {
                return Some(ts);
            }
        }
    }
    None
}

fn modified_after(path: &std::path::Path, cutoff: std::time::SystemTime) -> bool {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .map(|modified| modified >= cutoff)
        .unwrap_or(true)
}

fn timestamp_to_system_time(timestamp: Timestamp) -> std::time::SystemTime {
    if timestamp >= Timestamp::UNIX_EPOCH {
        let duration = timestamp.duration_since(Timestamp::UNIX_EPOCH);
        std::time::UNIX_EPOCH
            + std::time::Duration::new(duration.as_secs() as u64, duration.subsec_nanos() as u32)
    } else {
        std::time::UNIX_EPOCH
    }
}

fn persist_usage_history(paths: &PlatformPaths, report: &UsageReport) -> Result<()> {
    fs::create_dir_all(&paths.data_dir)?;
    let path = paths.data_dir.join("usage-history.json");
    let mut history = if path.is_file() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<Vec<UsageReport>>(&content).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    history.push(report.clone());
    if history.len() > 12 {
        history = history.split_off(history.len() - 12);
    }
    fs::write(path, serde_json::to_string_pretty(&history)?)?;
    Ok(())
}

fn print_usage_report(report: &UsageReport) {
    println!("=== Skills Janitor - Usage Report ===");
    println!("Period: last {} weeks", report.period_weeks);
    println!();
    let pct = percent(report.active_skills, report.total_skills);
    let unused_pct = percent(report.unused_skills, report.total_skills);
    println!(
        "Active skills: {} / {} ({}%)",
        report.active_skills, report.total_skills, pct
    );
    println!("Unused skills: {} ({}%)", report.unused_skills, unused_pct);
    if let Some(most) = &report.most_used {
        println!("Most used: {} ({} total)", most.name, most.count);
    }
    println!(
        "Recommendation: Remove or consolidate {} unused skills",
        report.unused_skills
    );
    println!();
    for skill in report.skills.iter().take(30) {
        println!("  {:<35} {:>4}  {}", skill.name, skill.count, skill.scope);
    }
}

fn percent(n: usize, d: usize) -> usize {
    if d == 0 {
        0
    } else {
        ((n as f64 / d as f64) * 100.0).round() as usize
    }
}
