use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::cli::InstallSkillsArgs;
use crate::domain::paths::skill_file_in;

const DEFAULT_TARGETS: [&str; 2] = ["claude", "agents"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct SkillInstallTarget {
    label: String,
    path: PathBuf,
}

pub fn run_install_skills(args: InstallSkillsArgs) -> Result<()> {
    let project = args
        .project
        .unwrap_or(std::env::current_dir().context("Could not determine current directory")?);
    let source = args.source.unwrap_or_else(|| project.join("skills"));
    let skill_dirs = discover_source_skills(&source)?;
    let targets = resolve_install_targets(&project, &args.target)?;

    for target in &targets {
        fs::create_dir_all(&target.path)
            .with_context(|| format!("create {}", target.path.display()))?;
        for skill_dir in &skill_dirs {
            let skill_name = skill_dir
                .file_name()
                .context("source skill directory is missing a folder name")?;
            let destination = target.path.join(skill_name);
            copy_skill_dir(skill_dir, &destination)?;
        }
    }

    println!(
        "Installed {} skills into {} target(s):",
        skill_dirs.len(),
        targets.len()
    );
    for target in &targets {
        println!(
            "- {} ({})",
            target.label,
            display_relative(&target.path, &project)
        );
    }

    Ok(())
}

fn discover_source_skills(source: &Path) -> Result<Vec<PathBuf>> {
    if !source.is_dir() {
        bail!("skill source directory not found: {}", source.display());
    }

    let mut skill_dirs = Vec::new();
    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() && skill_file_in(&path).is_some() {
            skill_dirs.push(path);
        }
    }

    skill_dirs.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));

    if skill_dirs.is_empty() {
        bail!(
            "no skill directories containing SKILL.md found in {}",
            source.display()
        );
    }

    Ok(skill_dirs)
}

fn resolve_install_targets(
    project: &Path,
    extra_targets: &[String],
) -> Result<Vec<SkillInstallTarget>> {
    let mut targets = Vec::new();
    let mut seen = HashSet::new();

    for target in DEFAULT_TARGETS {
        add_target(project, target, &mut targets, &mut seen)?;
    }

    for target in extra_targets {
        add_target(project, target, &mut targets, &mut seen)?;
    }

    Ok(targets)
}

fn add_target(
    project: &Path,
    target: &str,
    targets: &mut Vec<SkillInstallTarget>,
    seen: &mut HashSet<String>,
) -> Result<()> {
    let target = target.trim();
    if target.is_empty() {
        bail!("--target cannot be empty");
    }

    let resolved = resolve_target(project, target)?;
    let key = path_key(&resolved.path);
    if seen.insert(key) {
        targets.push(resolved);
    }

    Ok(())
}

fn resolve_target(project: &Path, target: &str) -> Result<SkillInstallTarget> {
    let normalized = target.trim().to_ascii_lowercase();
    let (label, path) = match normalized.as_str() {
        "claude" | ".claude" => ("claude".to_owned(), project.join(".claude").join("skills")),
        "agents" | ".agents" | "codex" => {
            ("agents".to_owned(), project.join(".agents").join("skills"))
        }
        "kiro" | ".kiro" => ("kiro".to_owned(), project.join(".kiro").join("skills")),
        _ if is_path_like(target) => {
            let path = resolve_path_target(project, target)?;
            (target.to_owned(), path)
        }
        _ => (
            target.to_owned(),
            project.join(format!(".{target}")).join("skills"),
        ),
    };

    Ok(SkillInstallTarget { label, path })
}

fn is_path_like(target: &str) -> bool {
    target.starts_with('.')
        || target.starts_with('~')
        || target.contains('/')
        || target.contains('\\')
        || Path::new(target).is_absolute()
}

fn resolve_path_target(project: &Path, target: &str) -> Result<PathBuf> {
    let path = expand_tilde(target)?;
    let path = if path.is_absolute() {
        path
    } else {
        project.join(path)
    };

    if path
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.eq_ignore_ascii_case("skills"))
    {
        Ok(path)
    } else {
        Ok(path.join("skills"))
    }
}

fn expand_tilde(target: &str) -> Result<PathBuf> {
    if target == "~" {
        return dirs::home_dir().context("Could not determine home directory for ~ target");
    }

    if let Some(rest) = target
        .strip_prefix("~/")
        .or_else(|| target.strip_prefix("~\\"))
    {
        let home = dirs::home_dir().context("Could not determine home directory for ~ target")?;
        return Ok(home.join(rest));
    }

    Ok(PathBuf::from(target))
}

fn copy_skill_dir(source: &Path, destination: &Path) -> Result<()> {
    reject_symlink(destination)?;
    if destination.exists() && !destination.is_dir() {
        bail!(
            "destination exists but is not a directory: {}",
            destination.display()
        );
    }

    fs::create_dir_all(destination).with_context(|| format!("create {}", destination.display()))?;
    copy_dir_contents(source, destination)
}

fn copy_dir_contents(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if file_type.is_symlink() {
            bail!(
                "refusing to install symlinked skill content: {}",
                source_path.display()
            );
        }

        reject_symlink(&destination_path)?;
        if file_type.is_dir() {
            if destination_path.exists() && !destination_path.is_dir() {
                bail!(
                    "destination exists but is not a directory: {}",
                    destination_path.display()
                );
            }
            fs::create_dir_all(&destination_path)
                .with_context(|| format!("create {}", destination_path.display()))?;
            copy_dir_contents(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            if destination_path.exists() && destination_path.is_dir() {
                bail!(
                    "destination exists but is a directory: {}",
                    destination_path.display()
                );
            }
            fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "copy {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn reject_symlink(path: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.file_type().is_symlink()
    {
        bail!("refusing to write through symlink: {}", path.display());
    }

    Ok(())
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn display_relative(path: &Path, project: &Path) -> String {
    path.strip_prefix(project)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_and_extra_targets_without_duplicates() {
        let project = Path::new("/repo");
        let targets = resolve_install_targets(project, &["kiro".into(), "claude".into()])
            .expect("targets should resolve");

        assert_eq!(
            targets
                .iter()
                .map(|target| target.label.as_str())
                .collect::<Vec<_>>(),
            vec!["claude", "agents", "kiro"]
        );
        assert_eq!(targets[0].path, project.join(".claude").join("skills"));
        assert_eq!(targets[1].path, project.join(".agents").join("skills"));
        assert_eq!(targets[2].path, project.join(".kiro").join("skills"));
    }

    #[test]
    fn resolves_unknown_target_names_to_hidden_skill_roots() {
        let target = resolve_target(Path::new("/repo"), "cursor").expect("target should resolve");

        assert_eq!(target.label, "cursor");
        assert_eq!(
            target.path,
            Path::new("/repo").join(".cursor").join("skills")
        );
    }

    #[test]
    fn path_targets_keep_explicit_skills_suffix() {
        let target =
            resolve_target(Path::new("/repo"), ".custom/skills").expect("target should resolve");

        assert_eq!(
            target.path,
            Path::new("/repo").join(".custom").join("skills")
        );
    }
}
