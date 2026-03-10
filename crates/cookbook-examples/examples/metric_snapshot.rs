//! System metric collection snapshot.
//!
//! Collects system metrics (CPU, memory, disk, load) from /proc and
//! displays them alongside recipe validation timing. Demonstrates how
//! forjar's metric collector integrates with recipe operations for
//! performance monitoring and capacity planning.
//!
//! Usage: `cargo run --example metric_snapshot`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let root = match cookbook_examples::find_project_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    eprintln!("--- Metric Snapshot ---\n");

    // Step 1: Collect system metrics from /proc
    eprintln!("Step 1: System metrics");
    collect_system_metrics();

    // Step 2: Time recipe validation across all recipes and report metrics
    let recipes_dir = root.join("recipes");
    let files = match cookbook_examples::collect_yaml_files(&recipes_dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    if files.is_empty() {
        eprintln!("no recipes found");
        return ExitCode::FAILURE;
    }

    eprintln!("\nStep 2: Validation timing ({} recipes)", files.len());
    let mut timings_ms: Vec<u64> = Vec::new();
    let mut failures = 0u32;

    let overall_start = std::time::Instant::now();
    for file in &files {
        let start = std::time::Instant::now();
        let result = Command::new("forjar")
            .args(["validate", "-f", &file.display().to_string()])
            .output();
        let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        match result {
            Ok(output) if output.status.success() => {
                timings_ms.push(elapsed_ms);
            }
            Ok(_) => {
                failures += 1;
                timings_ms.push(elapsed_ms);
            }
            Err(_) => {
                failures += 1;
            }
        }
    }
    let overall_ms = u64::try_from(overall_start.elapsed().as_millis()).unwrap_or(u64::MAX);

    // Step 3: Report aggregate metrics
    eprintln!("\nStep 3: Aggregate metrics");
    if !timings_ms.is_empty() {
        timings_ms.sort_unstable();
        let count = timings_ms.len();
        let sum: u64 = timings_ms.iter().sum();
        let mean = sum / count as u64;
        let p50 = timings_ms[count / 2];
        let p95_idx = count.saturating_sub(1).min(count * 95 / 100);
        let p95 = timings_ms[p95_idx];
        let max = timings_ms[count - 1];

        eprintln!("  recipes:     {count}");
        eprintln!("  failures:    {failures}");
        eprintln!("  total_ms:    {overall_ms}");
        eprintln!("  mean_ms:     {mean}");
        eprintln!("  p50_ms:      {p50}");
        eprintln!("  p95_ms:      {p95}");
        eprintln!("  max_ms:      {max}");
    }

    // Step 4: Post-validation system metrics (show delta)
    eprintln!("\nStep 4: Post-validation system metrics");
    collect_system_metrics();

    eprintln!("\n--- Metric snapshot complete ---");

    if failures > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Collect and print system metrics from /proc (Linux) or basic fallbacks.
fn collect_system_metrics() {
    // CPU from /proc/stat
    if let Ok(content) = std::fs::read_to_string("/proc/stat") {
        if let Some(pct) = parse_cpu_percent(&content) {
            eprintln!("  cpu_percent:         {pct:.1}%");
        }
    }

    // Memory from /proc/meminfo
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        if let Some((used_pct, avail_mb)) = parse_memory_info(&content) {
            eprintln!("  memory_percent:      {used_pct:.1}%");
            eprintln!("  memory_available_mb: {avail_mb:.0}");
        }
    }

    // Load average from /proc/loadavg
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(load) = content
            .split_whitespace()
            .next()
            .and_then(|v| v.parse::<f64>().ok())
        {
            eprintln!("  load_1m:             {load:.2}");
        }
    }

    // Disk usage via df
    if let Ok(output) = Command::new("df").args(["-P", "/"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(pct) = parse_disk_percent(&stdout) {
                eprintln!("  disk_percent:        {pct:.0}%");
            }
        }
    }
}

/// Parse CPU busy percentage from /proc/stat content.
fn parse_cpu_percent(content: &str) -> Option<f64> {
    let line = content.lines().find(|l| l.starts_with("cpu "))?;
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|f| f.parse().ok())
        .collect();
    if fields.len() < 4 {
        return None;
    }
    let idle = fields[3] + fields.get(4).copied().unwrap_or(0);
    let total: u64 = fields.iter().sum();
    if total == 0 {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let pct = (total - idle) as f64 / total as f64 * 100.0;
    Some(pct)
}

/// Parse memory info from `/proc/meminfo`: (`used_percent`, `available_mb`).
fn parse_memory_info(content: &str) -> Option<(f64, f64)> {
    let total_kb = extract_meminfo_kb(content, "MemTotal:")?;
    let available_kb = extract_meminfo_kb(content, "MemAvailable:")?;
    if total_kb == 0 {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let used_pct = (total_kb - available_kb) as f64 / total_kb as f64 * 100.0;
    #[allow(clippy::cast_precision_loss)]
    let avail_mb = available_kb as f64 / 1024.0;
    Some((used_pct, avail_mb))
}

fn extract_meminfo_kb(content: &str, key: &str) -> Option<u64> {
    content
        .lines()
        .find(|l| l.starts_with(key))
        .and_then(|line| line.split_whitespace().nth(1).and_then(|v| v.parse().ok()))
}

/// Parse disk usage percentage from `df -P` output.
fn parse_disk_percent(output: &str) -> Option<f64> {
    let data_line = output.lines().nth(1)?;
    let pct_field = data_line.split_whitespace().nth(4)?;
    pct_field.trim_end_matches('%').parse().ok()
}
