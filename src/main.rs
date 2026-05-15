use anyhow::Result;

mod analysis;
mod cli;
mod commands;
mod domain;
mod infra;

fn main() -> Result<()> {
    cli::run()
}
