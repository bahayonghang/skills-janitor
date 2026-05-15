use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::cli::{CompareArgs, PrecheckArgs, SearchArgs};
use crate::dupes;
use crate::frontmatter;
use crate::inventory;
use crate::output;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub url: String,
    pub description: String,
    pub stars: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchReport {
    pub keyword: String,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompareReport {
    pub skill: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub alternatives: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecheckOverlap {
    pub name: String,
    pub scope: String,
    pub similarity: u8,
    pub common_keywords: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecheckReport {
    pub source: String,
    pub new_skill: NewSkill,
    pub verdict: String,
    pub verdict_message: String,
    pub overlaps: Vec<PrecheckOverlap>,
    pub installed_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewSkill {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}

pub fn run_search(args: SearchArgs) -> Result<()> {
    let report = search_github(&args.keyword, args.limit)?;
    if args.json {
        output::print_json(&report)
    } else {
        print_search_report(&report);
        Ok(())
    }
}

pub fn run_compare(args: CompareArgs) -> Result<()> {
    let Some((_path, fm)) = inventory::find_installed_skill(&args.skill_name)? else {
        bail!(
            "Skill '{}' not found in any skill directory",
            args.skill_name
        );
    };
    let mut keywords = dupes::keyword_vec(&fm.description);
    if keywords.is_empty() {
        keywords.push(args.skill_name.clone());
    }
    let query = keywords
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let search = search_github(&query, 10).unwrap_or(SearchReport {
        keyword: query,
        results: Vec::new(),
    });
    let report = CompareReport {
        skill: args.skill_name,
        description: fm.description,
        keywords,
        alternatives: search.results,
    };
    if args.json {
        output::print_json(&report)
    } else {
        print_compare_report(&report);
        Ok(())
    }
}

pub fn run_precheck(args: PrecheckArgs) -> Result<()> {
    let report = build_precheck_report(&args.source)?;
    if args.json {
        output::print_json(&report)
    } else {
        print_precheck_report(&report);
        Ok(())
    }
}

fn search_github(keyword: &str, limit: usize) -> Result<SearchReport> {
    let query = format!("{} filename:SKILL.md", keyword);
    let url = format!(
        "https://api.github.com/search/code?q={}&per_page={}",
        url_encode(&query),
        limit.min(50)
    );
    let value = github_get_json(&url)
        .with_context(|| "GitHub search failed; set GITHUB_TOKEN if rate-limited")?;
    let mut results = Vec::new();
    if let Some(items) = value.get("items").and_then(|v| v.as_array()) {
        for item in items.iter().take(limit) {
            let html_url = item
                .get("html_url")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let repo = item.get("repository").and_then(|v| v.as_object());
            let repo_name = repo
                .and_then(|r| r.get("full_name"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let stars = repo
                .and_then(|r| r.get("stargazers_count"))
                .and_then(|v| v.as_u64())
                .unwrap_or_default();
            results.push(SearchResult {
                name: repo_name.to_string(),
                url: html_url.to_string(),
                description: repo
                    .and_then(|r| r.get("description"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                stars,
            });
        }
    }
    Ok(SearchReport {
        keyword: keyword.to_string(),
        results,
    })
}

fn github_get_json(url: &str) -> Result<serde_json::Value> {
    let mut request = ureq::get(url)
        .header("User-Agent", "skills-janitor")
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN")
        && !token.trim().is_empty()
    {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let mut response = request.call()?;
    Ok(response.body_mut().read_json()?)
}

fn url_encode(input: &str) -> String {
    input
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![b as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}

pub fn build_precheck_report(source: &str) -> Result<PrecheckReport> {
    let content = load_skill_source(source)?;
    let fm = frontmatter::parse_content(&content);
    let name = if fm.name.is_empty() {
        infer_name_from_source(source)
    } else {
        fm.name
    };
    let description = fm.description;
    let mut new_keywords = dupes::extract_keywords(&description);
    for word in name.replace('-', " ").split_whitespace() {
        if word.chars().count() > 2 {
            new_keywords.insert(word.to_lowercase());
        }
    }
    let inventory = inventory::build_inventory()?;
    let installed = dupes::installed_keyword_map(&inventory.skills);
    let mut overlaps = Vec::new();
    for (skill_name, scope, keywords, skill_desc) in &installed {
        let similarity = dupes::similarity_percent(&new_keywords, keywords);
        if similarity > 15 {
            overlaps.push(PrecheckOverlap {
                name: skill_name.clone(),
                scope: scope.clone(),
                similarity,
                common_keywords: dupes::common_keywords(&new_keywords, keywords, 10),
                description: skill_desc.clone(),
            });
        }
    }
    overlaps.sort_by_key(|overlap| std::cmp::Reverse(overlap.similarity));
    let (verdict, verdict_message) = if overlaps.first().is_some_and(|o| o.similarity >= 60) {
        ("HIGH_OVERLAP", "High overlap detected - likely duplicate")
    } else if overlaps.first().is_some_and(|o| o.similarity >= 30) {
        (
            "MODERATE_OVERLAP",
            "Moderate overlap - review before installing",
        )
    } else {
        ("SAFE", "No significant overlap - safe to install")
    };
    let mut keywords = new_keywords.into_iter().collect::<Vec<_>>();
    keywords.sort();
    Ok(PrecheckReport {
        source: source.to_string(),
        new_skill: NewSkill {
            name,
            description,
            keywords,
        },
        verdict: verdict.to_string(),
        verdict_message: verdict_message.to_string(),
        overlaps: overlaps.into_iter().take(10).collect(),
        installed_count: installed.len(),
    })
}

fn load_skill_source(source: &str) -> Result<String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let raw = github_raw_url(source);
        let mut request = ureq::get(&raw)
            .header("User-Agent", "skills-janitor")
            .header("Accept", "text/plain");
        if let Ok(token) = std::env::var("GITHUB_TOKEN")
            && !token.trim().is_empty()
        {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }
        let mut response = request.call()?;
        return Ok(response.body_mut().read_to_string()?);
    }
    let path = Path::new(source);
    let skill_file = if path.is_dir() {
        crate::paths::skill_file_in(path)
            .with_context(|| format!("No SKILL.md found in {source}"))?
    } else {
        PathBuf::from(path)
    };
    Ok(fs::read_to_string(skill_file)?)
}

fn github_raw_url(url: &str) -> String {
    if url.contains("raw.githubusercontent.com") {
        return url.to_string();
    }
    url.replace("https://github.com/", "https://raw.githubusercontent.com/")
        .replace("/blob/", "/")
        .replace("/tree/", "/")
        .trim_end_matches('/')
        .to_string()
        + if url.ends_with("SKILL.md") || url.ends_with("Skill.md") {
            ""
        } else {
            "/SKILL.md"
        }
}

fn infer_name_from_source(source: &str) -> String {
    source
        .trim_end_matches('/')
        .split(['/', '\\'])
        .next_back()
        .unwrap_or("unknown-skill")
        .trim_end_matches(".md")
        .to_string()
}

fn print_search_report(report: &SearchReport) {
    println!("=== Skills Janitor - GitHub Search ===");
    println!("Keyword: {}", report.keyword);
    println!();
    if report.results.is_empty() {
        println!("No results found or GitHub API returned no items.");
    }
    for result in &report.results {
        println!("  {} ({} stars)", result.name, result.stars);
        println!("    {}", result.url);
        if !result.description.is_empty() {
            println!("    {}", result.description);
        }
    }
}

fn print_compare_report(report: &CompareReport) {
    println!("=== Skills Janitor - Market Comparison ===");
    println!("Skill: {}", report.skill);
    println!("Description: {}", report.description);
    println!(
        "Keywords: {}",
        report
            .keywords
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    println!("Alternatives:");
    for result in &report.alternatives {
        println!("  {} ({} stars)", result.name, result.stars);
        println!("    {}", result.url);
    }
}

fn print_precheck_report(report: &PrecheckReport) {
    println!("=== Skills Janitor - Pre-Install Check ===");
    println!();
    println!("  Checking: {}", report.new_skill.name);
    println!("  Source: {}", report.source);
    println!(
        "  Description: {}",
        truncate(&report.new_skill.description, 120)
    );
    println!(
        "  Keywords: {}",
        report
            .new_skill
            .keywords
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    println!("  Scanned {} installed skills", report.installed_count);
    println!();
    if report.overlaps.is_empty() {
        println!("  No overlaps found with installed skills.");
        println!();
    } else {
        let high: Vec<_> = report
            .overlaps
            .iter()
            .filter(|o| o.similarity >= 60)
            .collect();
        let moderate: Vec<_> = report
            .overlaps
            .iter()
            .filter(|o| (30..60).contains(&o.similarity))
            .collect();
        let low: Vec<_> = report
            .overlaps
            .iter()
            .filter(|o| o.similarity < 30)
            .collect();
        if !high.is_empty() {
            println!("  --- HIGH OVERLAP (likely duplicates) ---");
            for o in high {
                println!("    [{}%] {} ({})", o.similarity, o.name, o.scope);
                println!(
                    "         Shared: {}",
                    o.common_keywords
                        .iter()
                        .take(6)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("         Existing desc: {}", truncate(&o.description, 80));
                println!();
            }
        }
        if !moderate.is_empty() {
            println!("  --- MODERATE OVERLAP (review before installing) ---");
            for o in moderate {
                println!("    [{}%] {} ({})", o.similarity, o.name, o.scope);
                println!(
                    "         Shared: {}",
                    o.common_keywords
                        .iter()
                        .take(6)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!();
            }
        }
        if !low.is_empty() {
            println!(
                "  --- LOW OVERLAP ({} skills with minor keyword matches) ---",
                low.len()
            );
            for o in low.into_iter().take(3) {
                println!("    [{}%] {}", o.similarity, o.name);
            }
            println!();
        }
    }
    println!("  VERDICT: {}", report.verdict_message);
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        value.to_string()
    } else {
        value.chars().take(limit).collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_github_tree_url_to_raw_skill() {
        assert_eq!(
            github_raw_url("https://github.com/u/r/tree/main/skills/demo"),
            "https://raw.githubusercontent.com/u/r/main/skills/demo/SKILL.md"
        );
    }
}
