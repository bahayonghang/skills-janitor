use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub has_frontmatter: bool,
    pub has_closing_delimiter: bool,
    pub name: String,
    pub description: String,
    pub version: String,
    pub disable_model_invocation: Option<bool>,
    #[serde(skip_serializing)]
    pub raw: String,
    #[serde(skip_serializing)]
    pub body: String,
    #[serde(skip_serializing)]
    pub line_count: usize,
}

impl Frontmatter {
    pub fn has_body(&self) -> bool {
        self.body.lines().any(|line| !line.trim().is_empty())
    }

    pub fn body_non_empty_lines(&self) -> usize {
        self.body
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
    }

    #[allow(dead_code)]
    pub fn fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        let mut current_key: Option<String> = None;
        for line in self.raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if !line.starts_with(char::is_whitespace) {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_string();
                    fields.insert(key.clone(), clean_scalar(value.trim()));
                    current_key = Some(key);
                }
            } else if let Some(key) = current_key.as_ref() {
                let entry = fields.entry(key.clone()).or_default();
                if !entry.is_empty() {
                    entry.push(' ');
                }
                entry.push_str(line.trim());
            }
        }
        fields
    }
}

pub fn parse_file(path: &Path) -> Result<Frontmatter> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_content(&content))
}

pub fn parse_content(content: &str) -> Frontmatter {
    let line_count = content.lines().count();
    let mut out = Frontmatter {
        line_count,
        ..Default::default()
    };

    let mut lines = content.lines();
    let Some(first) = lines.next() else {
        return out;
    };
    if first.trim() != "---" {
        out.body = content.to_string();
        return out;
    }

    out.has_frontmatter = true;
    let mut raw_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_body = false;

    for line in lines {
        if !in_body && line.trim() == "---" {
            out.has_closing_delimiter = true;
            in_body = true;
            continue;
        }
        if in_body {
            body_lines.push(line);
        } else {
            raw_lines.push(line);
        }
    }

    out.raw = raw_lines.join("\n");
    out.body = body_lines.join("\n");
    out.name = extract_field(&out.raw, "name").unwrap_or_default();
    out.description = extract_description(&out.raw).unwrap_or_default();
    out.version = extract_version(&out.raw).unwrap_or_default();
    out.disable_model_invocation = extract_field(&out.raw, "disable-model-invocation")
        .and_then(|value| value.parse::<bool>().ok());
    out
}

pub fn extract_field(raw: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    for line in raw.lines() {
        if line.starts_with(&prefix) {
            return Some(clean_scalar(line[prefix.len()..].trim()));
        }
    }
    None
}

pub fn extract_description(raw: &str) -> Option<String> {
    let mut lines = raw.lines().peekable();
    while let Some(line) = lines.next() {
        if let Some(rest) = line.strip_prefix("description:") {
            let value = rest.trim();
            if value == "|" || value == ">" || value.is_empty() {
                let mut chunks = Vec::new();
                while let Some(next) = lines.peek().copied() {
                    if next.starts_with(char::is_whitespace) {
                        chunks.push(next.trim().to_string());
                        lines.next();
                    } else {
                        break;
                    }
                }
                return Some(chunks.join(" ").trim().to_string());
            }
            return Some(clean_scalar(value));
        }
    }
    None
}

pub fn extract_version(raw: &str) -> Option<String> {
    let mut in_metadata = false;
    for line in raw.lines() {
        if line.starts_with("version:") {
            return Some(clean_scalar(line.trim_start_matches("version:").trim()));
        }
        if line.starts_with("metadata:") {
            in_metadata = true;
            continue;
        }
        if in_metadata {
            if line.starts_with(char::is_whitespace) {
                if let Some(rest) = line.trim().strip_prefix("version:") {
                    return Some(clean_scalar(rest.trim()));
                }
            } else {
                in_metadata = false;
            }
        }
    }
    None
}

pub fn clean_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

#[allow(dead_code)]
pub fn strip_frontmatter(content: &str) -> &str {
    let mut lines = content.lines();
    if !matches!(lines.next(), Some("---")) {
        return content;
    }
    let mut offset = 0;
    let mut seen_first = false;
    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        offset += line.len();
        if !seen_first {
            seen_first = true;
            continue;
        }
        if trimmed == "---" {
            return &content[offset..];
        }
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_metadata_version_and_chinese_description() {
        let fm = parse_content(
            "---\nname: test-skill\ndescription: \"当用户需要测试时使用\"\nmetadata:\n  version: \"1.2.3\"\n---\n# Body\n",
        );
        assert!(fm.has_frontmatter);
        assert_eq!(fm.name, "test-skill");
        assert_eq!(fm.description, "当用户需要测试时使用");
        assert_eq!(fm.version, "1.2.3");
        assert!(fm.has_body());
    }

    #[test]
    fn parses_block_description() {
        let fm = parse_content(
            "---\nname: block\ndescription: |\n  Use when alpha.\n  Also beta.\n---\nBody\n",
        );
        assert_eq!(fm.description, "Use when alpha. Also beta.");
    }
}
