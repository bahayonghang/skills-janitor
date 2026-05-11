use anyhow::Result;

pub mod cli;
pub mod dashboard;
pub mod dupes;
pub mod fix;
pub mod frontmatter;
pub mod fs_safety;
pub mod github;
pub mod inventory;
pub mod lint;
pub mod output;
pub mod paths;
pub mod tokens;
pub mod usage;

pub fn run() -> Result<()> {
    cli::run()
}
