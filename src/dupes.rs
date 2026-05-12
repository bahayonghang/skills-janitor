use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use regex::Regex;
use serde::Serialize;

use crate::inventory::{self, SkillRecord};

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub scope: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CanonicalSkill {
    pub name: String,
    pub description: String,
    pub real_path: String,
    pub locations: Vec<Location>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NameCollision {
    pub name: String,
    pub entries: Vec<CanonicalSkill>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescriptionOverlap {
    pub skill_a: String,
    pub skill_b: String,
    pub similarity: u8,
    pub common_keywords: Vec<String>,
    pub locations_a: Vec<Location>,
    pub locations_b: Vec<Location>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateReport {
    pub total_records: usize,
    pub unique_skill_files: usize,
    pub name_collisions: Vec<NameCollision>,
    pub description_overlaps: Vec<DescriptionOverlap>,
}

#[allow(dead_code)]
pub fn build_duplicate_report() -> Result<DuplicateReport> {
    let skills = inventory::all_skill_records()?;
    Ok(build_duplicate_report_from_records(&skills))
}

pub fn build_duplicate_report_from_records(records: &[SkillRecord]) -> DuplicateReport {
    let mut by_realpath: BTreeMap<String, CanonicalSkill> = BTreeMap::new();
    for record in records {
        if record.folder == "skills-janitor" || !record.has_skill_file {
            continue;
        }
        let key = if record.real_path.is_empty() {
            format!("{}/{}", record.scope, record.folder)
        } else {
            record.real_path.clone()
        };
        let entry = by_realpath.entry(key.clone()).or_insert(CanonicalSkill {
            name: record.folder.clone(),
            description: record.description.to_lowercase(),
            real_path: key.clone(),
            locations: Vec::new(),
        });
        entry.locations.push(Location {
            scope: record.scope.clone(),
            name: record.folder.clone(),
            path: record.path.clone(),
        });
    }

    let canonical: Vec<_> = by_realpath.into_values().collect();
    let mut by_name: BTreeMap<String, Vec<CanonicalSkill>> = BTreeMap::new();
    for skill in &canonical {
        by_name
            .entry(skill.name.clone())
            .or_default()
            .push(skill.clone());
    }
    let name_collisions = by_name
        .into_iter()
        .filter_map(|(name, entries)| {
            if entries.len() > 1 {
                Some(NameCollision { name, entries })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut description_overlaps = Vec::new();
    for i in 0..canonical.len() {
        let kw_i = extract_keywords(&canonical[i].description);
        if kw_i.is_empty() {
            continue;
        }
        for skill_j in canonical.iter().skip(i + 1) {
            let kw_j = extract_keywords(&skill_j.description);
            if kw_j.is_empty() {
                continue;
            }
            let common: HashSet<_> = kw_i.intersection(&kw_j).cloned().collect();
            let union: HashSet<_> = kw_i.union(&kw_j).cloned().collect();
            let similarity = if union.is_empty() {
                0.0
            } else {
                common.len() as f64 / union.len() as f64
            };
            if similarity > 0.3 {
                let mut common_keywords = common.into_iter().collect::<Vec<_>>();
                common_keywords.sort();
                common_keywords.truncate(10);
                description_overlaps.push(DescriptionOverlap {
                    skill_a: canonical[i].name.clone(),
                    skill_b: skill_j.name.clone(),
                    similarity: (similarity * 100.0).round() as u8,
                    common_keywords,
                    locations_a: canonical[i].locations.clone(),
                    locations_b: skill_j.locations.clone(),
                });
            }
        }
    }
    description_overlaps.sort_by_key(|overlap| std::cmp::Reverse(overlap.similarity));

    DuplicateReport {
        total_records: records.len(),
        unique_skill_files: canonical.len(),
        name_collisions,
        description_overlaps,
    }
}

pub fn extract_keywords(text: &str) -> HashSet<String> {
    let stop_words = stop_words();
    let regex = Regex::new(r"[A-Za-z]+|[\p{Han}]{2,}").expect("valid keyword regex");
    regex
        .find_iter(&text.to_lowercase())
        .map(|m| m.as_str().to_string())
        .filter(|word| !stop_words.contains(word.as_str()) && word.chars().count() > 2)
        .collect()
}

pub fn similarity_percent(a: &HashSet<String>, b: &HashSet<String>) -> u8 {
    let union = a.union(b).count();
    if union == 0 {
        return 0;
    }
    ((a.intersection(b).count() as f64 / union as f64) * 100.0).round() as u8
}

fn stop_words() -> HashSet<&'static str> {
    [
        "use", "when", "the", "user", "wants", "to", "or", "and", "a", "an", "this", "skill",
        "also", "that", "for", "with", "in", "on", "of", "is", "are", "it", "be", "as", "at", "by",
        "from", "their", "they", "has", "have", "do", "does", "can", "will", "about", "not", "but",
        "if", "its", "into", "your", "you", "how", "what", "which", "any", "all", "each", "every",
        "both", "more", "most", "other", "some", "such", "than", "too", "very", "just", "only",
        "own", "same", "mentions", "says", "asks", "help", "create", "make", "build", "improve",
        "optimize", "review", "write", "generate", "set", "up",
    ]
    .into_iter()
    .collect()
}

pub fn keyword_vec(text: &str) -> Vec<String> {
    let mut words: Vec<_> = extract_keywords(text).into_iter().collect();
    words.sort();
    words
}

pub fn print_duplicate_report(report: &DuplicateReport) {
    println!("=== Skills Janitor - Duplicate Detection ===");
    println!();
    println!("Total skill records: {}", report.total_records);
    println!(
        "Unique skill files (after symlink dedup): {}",
        report.unique_skill_files
    );
    println!();

    if !report.name_collisions.is_empty() {
        println!("--- Name Collisions ---");
        println!(
            "Found {} skill name(s) at multiple distinct paths:\n",
            report.name_collisions.len()
        );
        for collision in &report.name_collisions {
            println!("  {}", collision.name);
            for entry in &collision.entries {
                let mut scopes = entry
                    .locations
                    .iter()
                    .map(|loc| loc.scope.clone())
                    .collect::<Vec<_>>();
                scopes.sort();
                scopes.dedup();
                println!("    [{}] {}", scopes.join(", "), entry.real_path);
            }
            println!();
        }
    }

    if !report.description_overlaps.is_empty() {
        println!("--- Description Overlap (Jaccard > 30%) ---");
        println!(
            "Found {} potential overlap(s):\n",
            report.description_overlaps.len()
        );
        for overlap in &report.description_overlaps {
            println!(
                "  [{}%] {} <-> {}",
                overlap.similarity, overlap.skill_a, overlap.skill_b
            );
            println!(
                "       Shared keywords: {}",
                overlap.common_keywords.join(", ")
            );
            println!();
        }
    }

    if report.name_collisions.is_empty() && report.description_overlaps.is_empty() {
        println!("No obvious duplicates found.");
    }
}

pub fn common_keywords(a: &HashSet<String>, b: &HashSet<String>, limit: usize) -> Vec<String> {
    let mut common: Vec<_> = a.intersection(b).cloned().collect();
    common.sort();
    common.truncate(limit);
    common
}

pub fn installed_keyword_map(
    records: &[SkillRecord],
) -> Vec<(String, String, HashSet<String>, String)> {
    let mut out = Vec::new();
    for record in records {
        if !record.has_skill_file || record.folder == "skills-janitor" {
            continue;
        }
        let mut keywords = extract_keywords(&record.description);
        for word in record.folder.replace('-', " ").split_whitespace() {
            if word.chars().count() > 2 {
                keywords.insert(word.to_lowercase());
            }
        }
        out.push((
            record.folder.clone(),
            record.scope.clone(),
            keywords,
            record.description.clone(),
        ));
    }
    out
}

#[allow(dead_code)]
pub fn installed_name_counts(records: &[SkillRecord]) -> HashMap<String, usize> {
    records.iter().fold(HashMap::new(), |mut acc, record| {
        *acc.entry(record.folder.clone()).or_insert(0) += 1;
        acc
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_overlap_keywords() {
        let a = extract_keywords("Use when writing Rust command line tools");
        let b = extract_keywords("Use when building Rust CLI tools");
        assert!(similarity_percent(&a, &b) > 20);
        assert!(a.contains("rust"));
    }
}
