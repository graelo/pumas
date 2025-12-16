use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VM_STAT MEMORY TEST ===");

    // Test vm_stat command directly
    let output = Command::new("vm_stat").output()?;
    let output_str = String::from_utf8(output.stdout)?;

    println!("Raw vm_stat output:");
    println!("{}", output_str);

    // Parse key values using the same approach as the main implementation
    use std::collections::HashMap;

    let mut page_size = 4096u64;
    let mut values = HashMap::new();

    for line in output_str.lines() {
        if let Some(ps) = line.strip_prefix("Mach Virtual Memory Statistics: (page size of ") {
            if let Some(size_str) = ps.strip_suffix(" bytes)") {
                page_size = size_str.parse().unwrap_or(4096);
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

    // Extract values using the same field names as the main implementation
    let pages_active = values.get("Pages active").copied().unwrap_or(0);
    let pages_inactive = values.get("Pages inactive").copied().unwrap_or(0);
    let pages_wired = values.get("Pages wired down").copied().unwrap_or(0);
    let pages_file_backed = values.get("File-backed pages").copied().unwrap_or(0);
    let pages_anonymous = values.get("Anonymous pages").copied().unwrap_or(0);
    let pages_compressed = values
        .get("Pages occupied by compressor")
        .copied()
        .unwrap_or(0);
    let pages_free = values.get("Pages free").copied().unwrap_or(0);

    println!("\n=== PARSED VALUES ===");
    println!("Page size: {} bytes", page_size);
    println!("Anonymous pages: {}", pages_anonymous);
    println!("Wired pages: {}", pages_wired);
    println!("Compressed pages: {}", pages_compressed);
    println!("File-backed pages: {}", pages_file_backed);

    let app_memory_gb = (pages_anonymous * page_size) as f64 / (1024.0 * 1024.0 * 1024.0);
    let wired_memory_gb = (pages_wired * page_size) as f64 / (1024.0 * 1024.0 * 1024.0);
    let compressed_gb = (pages_compressed * page_size) as f64 / (1024.0 * 1024.0 * 1024.0);
    let cached_files_gb = (pages_file_backed * page_size) as f64 / (1024.0 * 1024.0 * 1024.0);

    // Total memory calculation (same as main implementation)
    let total_gb = ((pages_free + pages_active + pages_inactive + pages_wired) * page_size) as f64
        / (1024.0 * 1024.0 * 1024.0);

    // Activity Monitor approximation (same formula as main implementation)
    let activity_monitor_approx = app_memory_gb + wired_memory_gb + compressed_gb;

    println!("\n=== CALCULATED VALUES ===");
    println!("Total memory: {:.1} GB", total_gb);
    println!("App memory (anonymous): {:.1} GB", app_memory_gb);
    println!("Wired memory: {:.1} GB", wired_memory_gb);
    println!("Compressed: {:.1} GB", compressed_gb);
    println!("Cached files: {:.1} GB", cached_files_gb);
    println!(
        "Activity Monitor approximation: {:.1} GB",
        activity_monitor_approx
    );

    println!("\n=== ANALYSIS ===");
    println!("Our approximation: {:.1} GB", activity_monitor_approx);
    println!(
        "Formula: Anonymous + Wired + Compressed = {:.1} + {:.1} + {:.1}",
        app_memory_gb, wired_memory_gb, compressed_gb
    );
    println!();
    println!("Note: This approximation may differ from Activity Monitor's displayed");
    println!("'Memory Used' value, as Activity Monitor includes additional factors");
    println!("and uses different accounting methods. The approximation is most");
    println!("accurate when the system has been running for a while with active");
    println!("memory compression.");

    Ok(())
}
