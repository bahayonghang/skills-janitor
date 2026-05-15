use std::collections::HashMap;

use anyhow::Result;

use crate::analysis::usage::{self, UsageReport};
use crate::domain::inventory::{self, Inventory, ScanSnapshot};
use crate::domain::paths::PlatformPaths;

pub struct JanitorContext {
    pub paths: PlatformPaths,
    pub scan: ScanSnapshot,
    usage_cache: HashMap<u32, UsageReport>,
}

impl JanitorContext {
    pub fn with_paths(paths: PlatformPaths) -> Result<Self> {
        let scan = inventory::scan_snapshot_with_paths(&paths)?;
        Ok(Self {
            paths,
            scan,
            usage_cache: HashMap::new(),
        })
    }

    pub fn inventory(&self) -> Inventory {
        self.scan.inventory()
    }

    pub fn usage_report(&mut self, weeks: u32) -> Result<&UsageReport> {
        if !self.usage_cache.contains_key(&weeks) {
            let report = usage::build_usage_report_from_snapshot(&self.paths, &self.scan, weeks)?;
            self.usage_cache.insert(weeks, report);
        }
        Ok(self
            .usage_cache
            .get(&weeks)
            .expect("usage report was just cached"))
    }

    #[cfg(test)]
    pub fn cached_usage_report_count(&self) -> usize {
        self.usage_cache.len()
    }
}
