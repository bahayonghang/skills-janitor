use anyhow::Result;
use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[allow(dead_code)]
pub fn human_bool(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
