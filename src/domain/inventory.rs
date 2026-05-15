use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use jiff::Timestamp;
use serde::Serialize;
use walkdir::WalkDir;

use crate::domain::frontmatter::{self, Frontmatter};
use crate::domain::paths::{self, PlatformPaths, SkillDir};
use crate::infra::output;

#[derive(Debug, Clone, Serialize)]
pub struct SkillRecord {
    pub folder: String,
    pub scope: String,
    pub platform: String,
    pub path: String,
    pub real_path: String,
    pub is_symlink: bool,
    pub symlink_target: String,
    pub has_skill_file: bool,
    pub name: String,
    pub description: String,
    pub version: String,
    pub has_frontmatter: bool,
    pub has_body: bool,
    pub line_count: usize,
    pub extra_files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginRecord {
    pub name: String,
    pub source: String,
    pub version: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub scope: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandRecord {
    pub name: String,
    pub scope: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Inventory {
    pub scan_date: String,
    pub skills: Vec<SkillRecord>,
    pub plugins: Vec<PluginRecord>,
    pub commands: Vec<CommandRecord>,
    pub broken_symlinks: usize,
}

#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub record: SkillRecord,
    pub dir: PathBuf,
    pub skill_file: Option<PathBuf>,
    pub frontmatter: Frontmatter,
    pub skill_file_content: Option<String>,
    pub skill_file_chars: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ScanSnapshot {
    pub scan_date: String,
    pub skills: Vec<SkillEntry>,
    pub plugins: Vec<PluginRecord>,
    pub commands: Vec<CommandRecord>,
    pub broken_symlinks: usize,
}

impl ScanSnapshot {
    pub fn inventory(&self) -> Inventory {
        Inventory {
            scan_date: self.scan_date.clone(),
            skills: self
                .skills
                .iter()
                .map(|entry| entry.record.clone())
                .collect(),
            plugins: self.plugins.clone(),
            commands: self.commands.clone(),
            broken_symlinks: self.broken_symlinks,
        }
    }

    pub fn skill_records(&self) -> Vec<SkillRecord> {
        self.skills
            .iter()
            .map(|entry| entry.record.clone())
            .collect()
    }
}

pub fn run_scan(json: bool) -> Result<()> {
    let inventory = build_inventory()?;
    if json {
        output::print_json(&inventory)
    } else {
        print_scan_human(&inventory);
        Ok(())
    }
}

pub fn build_inventory() -> Result<Inventory> {
    let paths = PlatformPaths::detect()?;
    build_inventory_with_paths(&paths)
}

pub fn build_inventory_with_paths(paths: &PlatformPaths) -> Result<Inventory> {
    Ok(scan_snapshot_with_paths(paths)?.inventory())
}

pub fn scan_snapshot() -> Result<ScanSnapshot> {
    let paths = PlatformPaths::detect()?;
    scan_snapshot_with_paths(&paths)
}

pub fn scan_snapshot_with_paths(paths: &PlatformPaths) -> Result<ScanSnapshot> {
    let mut skills = Vec::new();
    for root in paths.skill_roots() {
        for dir in paths::iter_skill_dirs(&root.path)? {
            if dir.file_name().and_then(|n| n.to_str()) == Some("skills-janitor") {
                continue;
            }
            skills.push(scan_skill_entry(&root, &dir)?);
        }
    }

    Ok(ScanSnapshot {
        scan_date: Timestamp::now().to_string(),
        skills,
        plugins: scan_plugins(paths),
        commands: scan_commands(paths)?,
        broken_symlinks: count_broken_symlinks(&paths.claude_user_skills)?,
    })
}

#[cfg(test)]
pub fn scan_skill_dir(root: &SkillDir, dir: &Path) -> Result<SkillRecord> {
    Ok(scan_skill_entry(root, dir)?.record)
}

pub fn scan_skill_entry(root: &SkillDir, dir: &Path) -> Result<SkillEntry> {
    let folder = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let metadata = fs::symlink_metadata(dir).ok();
    let is_symlink = metadata
        .as_ref()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);
    let symlink_target = if is_symlink {
        match fs::read_link(dir) {
            Ok(target) if dir.exists() => target.display().to_string(),
            Ok(target) => format!("BROKEN:{}", target.display()),
            Err(_) => "broken".to_string(),
        }
    } else {
        String::new()
    };

    let skill_file = paths::skill_file_in(dir);
    let skill_file_content = match skill_file.as_ref() {
        Some(path) => fs::read_to_string(path).ok(),
        None => None,
    };
    let fm = match skill_file_content.as_deref() {
        Some(content) => frontmatter::parse_content(content),
        None => Frontmatter::default(),
    };
    let skill_file_chars = skill_file_content
        .as_ref()
        .map(|content| content.chars().count() as u64);

    let has_body = fm.has_body();

    let extra_files = if dir.is_dir() {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy();
                name != "SKILL.md" && name != "Skill.md" && name != ".DS_Store"
            })
            .count()
    } else {
        0
    };

    let record = SkillRecord {
        folder,
        scope: root.scope.as_str().to_string(),
        platform: root.platform.to_string(),
        path: dir.display().to_string(),
        real_path: paths::canonical_or_self(dir).display().to_string(),
        is_symlink,
        symlink_target,
        has_skill_file: skill_file.is_some(),
        name: fm.name.clone(),
        description: fm.description.clone(),
        version: fm.version.clone(),
        has_frontmatter: fm.has_frontmatter,
        has_body,
        line_count: fm.line_count,
        extra_files,
    };

    Ok(SkillEntry {
        record,
        dir: dir.to_path_buf(),
        skill_file,
        frontmatter: fm,
        skill_file_content,
        skill_file_chars,
    })
}

fn scan_plugins(paths: &PlatformPaths) -> Vec<PluginRecord> {
    let plugin_file = paths.claude_plugins.join("installed_plugins.json");
    let Ok(content) = fs::read_to_string(plugin_file) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Vec::new();
    };
    let mut records = Vec::new();
    if let Some(map) = value.get("plugins").and_then(|plugins| plugins.as_object()) {
        for (key, instances) in map {
            let (name, source) = split_plugin_key(key);
            let Some(instances) = instances.as_array() else {
                continue;
            };
            for instance in instances.iter().filter_map(|v| v.as_object()) {
                records.push(PluginRecord {
                    name: name.to_string(),
                    source: source.to_string(),
                    version: instance
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    scope: instance
                        .get("scope")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                });
            }
        }
    } else if let Some(array) = value.as_array() {
        for item in array.iter().filter_map(|v| v.as_object()) {
            records.push(PluginRecord {
                name: item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                source: item
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                version: item
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                scope: String::new(),
            });
        }
    }
    records
}

fn split_plugin_key(key: &str) -> (&str, &str) {
    key.rsplit_once('@').unwrap_or((key, ""))
}

fn scan_commands(paths: &PlatformPaths) -> Result<Vec<CommandRecord>> {
    let mut records = Vec::new();
    for (scope, dir) in [
        ("user", paths.claude_user_commands.as_path()),
        ("project", paths.claude_project_commands.as_path()),
    ] {
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            records.push(CommandRecord {
                name,
                scope: scope.to_string(),
                path: path.display().to_string(),
            });
        }
    }
    Ok(records)
}

pub fn count_broken_symlinks(dir: &Path) -> Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() && !entry.path().exists() {
            count += 1;
        }
    }
    Ok(count)
}

pub fn all_skill_records() -> Result<Vec<SkillRecord>> {
    Ok(scan_snapshot()?.skill_records())
}

pub fn find_installed_skill(name: &str) -> Result<Option<(PathBuf, Frontmatter)>> {
    let paths = PlatformPaths::detect()?;
    for root in paths.skill_roots() {
        let candidate = root.path.join(name);
        if !candidate.exists() {
            continue;
        }
        if let Some(skill_file) = paths::skill_file_in(&candidate) {
            let fm = frontmatter::parse_file(&skill_file)?;
            return Ok(Some((candidate, fm)));
        }
    }
    Ok(None)
}

pub fn print_scan_human(inventory: &Inventory) {
    println!("=== Skills Janitor - Inventory ===");
    println!("Scan date: {}", inventory.scan_date);
    println!("Skills: {}", inventory.skills.len());
    println!("Plugins: {}", inventory.plugins.len());
    println!("Commands: {}", inventory.commands.len());
    println!("Broken symlinks: {}", inventory.broken_symlinks);
    println!();
    for skill in &inventory.skills {
        println!(
            "  [{}] {} ({}) - {}",
            skill.scope,
            skill.folder,
            if skill.has_skill_file {
                "SKILL.md"
            } else {
                "no skill file"
            },
            if skill.description.is_empty() {
                "no description"
            } else {
                skill.description.as_str()
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::domain::paths::{SkillDir, SkillScope};

    #[test]
    fn scans_skill_record_with_metadata_version() {
        let dir = tempdir().unwrap();
        let skill = dir.path().join("example-skill");
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: example-skill\ndescription: Use when testing this skill.\nmetadata:\n  version: 9\n---\nBody line\n",
        )
        .unwrap();
        let root = SkillDir {
            scope: SkillScope::User,
            platform: "claude",
            path: dir.path().to_path_buf(),
        };
        let record = scan_skill_dir(&root, &skill).unwrap();
        assert_eq!(record.name, "example-skill");
        assert_eq!(record.version, "9");
        assert_eq!(record.extra_files, 0);
    }
}
