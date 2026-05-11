use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent", path.display()))?;
    fs::create_dir_all(parent)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    use std::io::Write as _;
    tmp.write_all(content.as_bytes())?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|err| anyhow::anyhow!("persist {}: {}", path.display(), err))?;
    Ok(())
}

pub fn ensure_within(path: &Path, root: &Path) -> Result<()> {
    let root = canonical_existing_or_parent(root)?;
    let candidate = canonical_existing_or_parent(path)?;
    if !candidate.starts_with(&root) {
        bail!(
            "Refusing to operate outside root: {} is not under {}",
            candidate.display(),
            root.display()
        );
    }
    Ok(())
}

pub fn canonical_existing_or_parent(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return Ok(path.canonicalize()?);
    }
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent", path.display()))?;
    Ok(parent
        .canonicalize()?
        .join(path.file_name().unwrap_or_default()))
}

pub fn is_plugin_or_marketplace_path(path: &Path) -> bool {
    let lower = path.to_string_lossy().replace('\\', "/").to_lowercase();
    lower.contains("/plugins/")
        || lower.contains("/sources/")
        || lower.contains("/plugins/cache/")
        || lower.contains("/marketplace/")
}

pub fn remove_empty_dir_checked(path: &Path, root: &Path) -> Result<()> {
    ensure_within(path, root)?;
    fs::remove_dir(path).with_context(|| format!("remove empty dir {}", path.display()))?;
    Ok(())
}

pub fn remove_file_checked(path: &Path, root: &Path) -> Result<()> {
    ensure_within(path, root)?;
    fs::remove_file(path).with_context(|| format!("remove file {}", path.display()))?;
    Ok(())
}
