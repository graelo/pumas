// Simple test that just runs vm_stat and sysinfo separately to check for ratio issues
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MEMORY RATIO TEST ===");

    // Test vm_stat directly
    let output = Command::new("vm_stat").output()?;
    let output_str = String::from_utf8(output.stdout)?;

    let mut page_size = 4096u64;
    let mut pages_anonymous = 0u64;
    let mut pages_wired = 0u64;
    let mut pages_compressed = 0u64;

    for line in output_str.lines() {
        if let Some(ps) = line.strip_prefix("Mach Virtual Memory Statistics: (page size of ") {
            if let Some(size_str) = ps.strip_suffix(" bytes)") {
                page_size = size_str.parse().unwrap_or(4096);
            }
        } else if line.contains("Anonymous pages:") {
            if let Some(val) = line.split(':').nth(1) {
                pages_anonymous = val.trim().trim_end_matches('.').parse().unwrap_or(0);
            }
        } else if line.contains("Pages wired down:") {
            if let Some(val) = line.split(':').nth(1) {
                pages_wired = val.trim().trim_end_matches('.').parse().unwrap_or(0);
            }
        } else if line.contains("Pages stored in compressor:") {
            if let Some(val) = line.split(':').nth(1) {
                pages_compressed = val.trim().trim_end_matches('.').parse().unwrap_or(0);
            }
        }
    }

    let vm_stat_memory_used = (pages_anonymous + pages_wired + pages_compressed) * page_size;

    // Test sysinfo total memory
    use sysinfo::{MemoryRefreshKind, System};
    let mut system = System::new();
    system.refresh_memory_specifics(MemoryRefreshKind::everything());
    let sysinfo_total = system.total_memory();

    let ram_ratio = vm_stat_memory_used as f64 / sysinfo_total as f64;

    println!(
        "VM_STAT Memory Used: {:.2} GB",
        vm_stat_memory_used as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!(
        "SYSINFO Total Memory: {:.2} GB",
        sysinfo_total as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!("Ratio: {:.3}", ram_ratio);

    if ram_ratio < 0.0 {
        println!("ERROR: Negative ratio detected!");
    } else if ram_ratio > 1.0 {
        println!("ERROR: Ratio > 1.0 detected! This would cause gauge issues.");
        println!("This happens when vm_stat memory_used > sysinfo total_memory");
    } else {
        println!("OK: Ratio is valid ({:.1}%)", ram_ratio * 100.0);
    }

    Ok(())
}
