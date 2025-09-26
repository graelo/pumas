//! VM statistics from the macOS `vm_stat` command.
//!
//! This provides memory statistics that are closer to what Activity Monitor shows.

use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Default)]
pub(crate) struct VmStats {
    pub page_size: u64,
    pub pages_active: u64,
    pub pages_inactive: u64,
    pub pages_wired: u64,
    pub pages_file_backed: u64,
    pub pages_anonymous: u64,
    pub pages_compressed: u64,
    pub pages_free: u64,
}

impl VmStats {
    /// Collect VM statistics by parsing `vm_stat` output.
    pub fn collect() -> Result<Self, Box<dyn std::error::Error>> {
        let output = Command::new("vm_stat").output()?;
        let output_str = String::from_utf8(output.stdout)?;

        let mut stats = VmStats::default();
        let mut values = HashMap::new();

        // Parse the output
        for line in output_str.lines() {
            if let Some(page_size) =
                line.strip_prefix("Mach Virtual Memory Statistics: (page size of ")
            {
                if let Some(size_str) = page_size.strip_suffix(" bytes)") {
                    stats.page_size = size_str.parse().unwrap_or(4096);
                }
            } else if line.contains(':') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value_str = parts[1].trim().trim_end_matches('.');
                    if let Ok(value) = value_str.parse::<u64>() {
                        values.insert(key, value);
                    }
                }
            }
        }

        // Extract the values we need
        stats.pages_active = values.get("Pages active").copied().unwrap_or(0);
        stats.pages_inactive = values.get("Pages inactive").copied().unwrap_or(0);
        stats.pages_wired = values.get("Pages wired down").copied().unwrap_or(0);
        stats.pages_file_backed = values.get("File-backed pages").copied().unwrap_or(0);
        stats.pages_anonymous = values.get("Anonymous pages").copied().unwrap_or(0);
        stats.pages_compressed = values
            .get("Pages stored in compressor")
            .copied()
            .unwrap_or(0);
        stats.pages_free = values.get("Pages free").copied().unwrap_or(0);

        Ok(stats)
    }

    /// Calculate memory usage closer to Activity Monitor's calculation.
    /// This approximates Activity Monitor's "Memory Used" by including:
    /// - Anonymous pages (app memory)
    /// - Wired pages (wired memory)
    /// - Compressed pages
    pub fn activity_monitor_memory_used(&self) -> u64 {
        let app_memory = self.pages_anonymous * self.page_size;
        let wired_memory = self.pages_wired * self.page_size;
        let compressed_memory = self.pages_compressed * self.page_size;

        app_memory + wired_memory + compressed_memory
    }

    /// Total physical memory.
    pub fn total_memory(&self) -> u64 {
        (self.pages_free + self.pages_active + self.pages_inactive + self.pages_wired)
            * self.page_size
    }
}
