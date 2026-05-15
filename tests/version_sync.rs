use std::fs;
use std::path::{Path, PathBuf};

const JANITOR_AUDIT_SKILL: &str = "skills/janitor-audit/SKILL.md";

#[test]
fn janitor_audit_skill_version_matches_package_version() {
    let root = manifest_dir();
    let cargo_version = read_package_version(&root.join("Cargo.toml"));
    let skill_version = read_janitor_audit_skill_version(&root.join(JANITOR_AUDIT_SKILL));

    assert_eq!(
        skill_version, cargo_version,
        "{JANITOR_AUDIT_SKILL} metadata.version must match Cargo.toml package.version"
    );
}

#[test]
#[ignore = "mutates the janitor-audit skill frontmatter to match Cargo.toml"]
fn sync_janitor_audit_skill_version() {
    let root = manifest_dir();
    let cargo_version = read_package_version(&root.join("Cargo.toml"));
    let skill_path = root.join(JANITOR_AUDIT_SKILL);
    let skill = fs::read_to_string(&skill_path).expect("failed to read janitor-audit skill");
    let updated = rewrite_frontmatter_version(&skill, &cargo_version);

    if updated != skill {
        fs::write(&skill_path, updated).expect("failed to write synchronized skill version");
    }

    assert_eq!(
        read_janitor_audit_skill_version(&skill_path),
        cargo_version,
        "{JANITOR_AUDIT_SKILL} metadata.version must match Cargo.toml after sync"
    );
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_package_version(path: &Path) -> String {
    let manifest = fs::read_to_string(path).expect("failed to read Cargo.toml");
    let mut in_package = false;

    for line in manifest.lines() {
        let trimmed = line.trim();

        if trimmed == "[package]" {
            in_package = true;
            continue;
        }

        if in_package && trimmed.starts_with('[') {
            break;
        }

        if in_package && let Some(version) = trimmed.strip_prefix("version = ") {
            return unquote(version).to_owned();
        }
    }

    panic!("Cargo.toml is missing [package].version");
}

fn read_janitor_audit_skill_version(path: &Path) -> String {
    let skill = fs::read_to_string(path).expect("failed to read janitor-audit skill");
    read_frontmatter_version(&skill)
}

fn read_frontmatter_version(skill: &str) -> String {
    for line in frontmatter_lines(skill) {
        if let Some(version) = line.trim().strip_prefix("version:") {
            return unquote(version.trim()).to_owned();
        }
    }

    panic!("{JANITOR_AUDIT_SKILL} frontmatter is missing metadata.version");
}

fn rewrite_frontmatter_version(skill: &str, cargo_version: &str) -> String {
    let had_trailing_newline = skill.ends_with('\n');
    let mut frontmatter_delimiters = 0;
    let mut replaced = false;
    let mut lines = Vec::new();

    for line in skill.lines() {
        if line.trim() == "---" {
            frontmatter_delimiters += 1;
            lines.push(line.to_owned());
            continue;
        }

        if !replaced && frontmatter_delimiters == 1 {
            let trimmed = line.trim_start();

            if trimmed.starts_with("version:") {
                let indent_len = line.len() - trimmed.len();
                let indent = &line[..indent_len];
                lines.push(format!("{indent}version: {cargo_version}"));
                replaced = true;
                continue;
            }
        }

        lines.push(line.to_owned());
    }

    assert!(
        replaced,
        "{JANITOR_AUDIT_SKILL} frontmatter is missing metadata.version"
    );

    let mut updated = lines.join("\n");
    if had_trailing_newline {
        updated.push('\n');
    }
    updated
}

fn frontmatter_lines(skill: &str) -> impl Iterator<Item = &str> {
    let mut delimiter_count = 0;

    skill.lines().filter(move |line| {
        if line.trim() == "---" {
            delimiter_count += 1;
            return false;
        }

        delimiter_count == 1
    })
}

fn unquote(value: &str) -> &str {
    value.trim().trim_matches('"').trim_matches('\'')
}
