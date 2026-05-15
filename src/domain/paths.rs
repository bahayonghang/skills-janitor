use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillScope {
    User,
    Project,
    CodexUser,
    CodexProject,
}

impl SkillScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Project => "project",
            Self::CodexUser => "codex-user",
            Self::CodexProject => "codex-project",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillDir {
    pub scope: SkillScope,
    pub platform: &'static str,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PlatformPaths {
    pub home: PathBuf,
    pub cwd: PathBuf,
    pub claude_user_skills: PathBuf,
    pub claude_project_skills: PathBuf,
    pub codex_user_skills: PathBuf,
    pub codex_project_skills: PathBuf,
    pub claude_user_commands: PathBuf,
    pub claude_project_commands: PathBuf,
    pub claude_plugins: PathBuf,
    pub data_dir: PathBuf,
}

impl PlatformPaths {
    pub fn detect() -> Result<Self> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let cwd = std::env::current_dir().context("Could not determine current directory")?;
        Ok(Self {
            claude_user_skills: home.join(".claude").join("skills"),
            claude_project_skills: cwd.join(".claude").join("skills"),
            codex_user_skills: home.join(".agents").join("skills"),
            codex_project_skills: cwd.join(".agents").join("skills"),
            claude_user_commands: home.join(".claude").join("commands"),
            claude_project_commands: cwd.join(".claude").join("commands"),
            claude_plugins: home.join(".claude").join("plugins"),
            data_dir: home
                .join(".claude")
                .join("skills")
                .join("skills-janitor")
                .join("data"),
            home,
            cwd,
        })
    }

    #[cfg(test)]
    pub fn from_home_and_cwd(home: impl Into<PathBuf>, cwd: impl Into<PathBuf>) -> Self {
        let home = home.into();
        let cwd = cwd.into();
        Self {
            claude_user_skills: home.join(".claude").join("skills"),
            claude_project_skills: cwd.join(".claude").join("skills"),
            codex_user_skills: home.join(".agents").join("skills"),
            codex_project_skills: cwd.join(".agents").join("skills"),
            claude_user_commands: home.join(".claude").join("commands"),
            claude_project_commands: cwd.join(".claude").join("commands"),
            claude_plugins: home.join(".claude").join("plugins"),
            data_dir: home
                .join(".claude")
                .join("skills")
                .join("skills-janitor")
                .join("data"),
            home,
            cwd,
        }
    }

    pub fn skill_roots(&self) -> Vec<SkillDir> {
        let mut roots = Vec::new();
        let mut seen_project_roots = HashSet::new();

        add_root(
            &mut roots,
            SkillScope::User,
            "claude",
            self.claude_user_skills.clone(),
        );
        add_root(
            &mut roots,
            SkillScope::CodexUser,
            "codex",
            self.codex_user_skills.clone(),
        );

        add_project_root(
            &mut roots,
            &mut seen_project_roots,
            SkillScope::Project,
            "claude",
            self.claude_project_skills.clone(),
        );
        add_project_root(
            &mut roots,
            &mut seen_project_roots,
            SkillScope::CodexProject,
            "codex",
            self.codex_project_skills.clone(),
        );

        roots
            .into_iter()
            .filter(|root| root.path.is_dir())
            .collect()
    }

    pub fn history_candidates(&self) -> Vec<PathBuf> {
        vec![
            self.home.join(".claude").join("history.jsonl"),
            self.home
                .join(".claude-account-personal")
                .join("history.jsonl"),
        ]
    }

    pub fn conversation_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.home.join(".claude").join("projects"),
            self.home.join(".claude-account-personal").join("projects"),
        ]
        .into_iter()
        .filter(|path| path.is_dir())
        .collect()
    }
}

fn add_root(roots: &mut Vec<SkillDir>, scope: SkillScope, platform: &'static str, path: PathBuf) {
    roots.push(SkillDir {
        scope,
        platform,
        path,
    });
}

fn add_project_root(
    roots: &mut Vec<SkillDir>,
    seen_project_roots: &mut HashSet<PathBuf>,
    scope: SkillScope,
    platform: &'static str,
    path: PathBuf,
) {
    if !path.is_dir() {
        return;
    }
    let real = canonical_or_self(&path);
    if roots
        .iter()
        .any(|existing| canonical_or_self(&existing.path) == real)
        || !seen_project_roots.insert(real)
    {
        return;
    }
    roots.push(SkillDir {
        scope,
        platform,
        path,
    });
}

pub fn canonical_or_self(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub fn skill_file_in(dir: &Path) -> Option<PathBuf> {
    let upper = dir.join("SKILL.md");
    if upper.is_file() {
        return Some(upper);
    }
    let title = dir.join("Skill.md");
    if title.is_file() {
        return Some(title);
    }
    None
}

pub fn iter_skill_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() || file_type.is_symlink() {
            dirs.push(path);
        }
    }
    dirs.sort_by_key(|p| p.file_name().map(|n| n.to_os_string()));
    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_default_roots() {
        let paths = PlatformPaths::from_home_and_cwd("/home/me", "/repo");
        assert!(paths.claude_user_skills.ends_with(".claude/skills"));
        assert!(paths.codex_project_skills.ends_with(".agents/skills"));
    }
}
