use anyhow::Result;

mod cli;
mod dashboard;
mod dupes;
mod fix;
mod frontmatter;
mod fs_safety;
mod github;
mod inventory;
mod lint;
mod output;
mod paths;
mod skill_install;
mod tokens;
mod usage;

fn main() -> Result<()> {
    cli::run()
}
