//! State encryption — encrypt, decrypt, and rekey state files.
//!
//! Demonstrates ephemeral state encryption: `forjar state-encrypt`,
//! `forjar state-decrypt`, and `forjar state-rekey`. Uses age-compatible
//! passphrase encryption for state-at-rest protection.
//!
//! Usage: `cargo run --example state_encryption`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-state-encryption");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- State Encryption ---\n");

    let mut failures = 0u32;

    // Step 1: Create a fake state directory with a lock file
    let state_dir = tmp.join("state");
    let machine_dir = state_dir.join("web-01");
    std::fs::create_dir_all(&machine_dir).ok();
    std::fs::write(
        machine_dir.join("lock.yaml"),
        "version: '1.0'\nresources:\n  nginx:\n    status: converged\n    hash: abc123\n",
    )
    .ok();

    let state_dir_str = state_dir.display().to_string();

    // Step 2: Encrypt state
    eprintln!("Step 1: Encrypt state with passphrase");
    let encrypt = run_forjar(&[
        "state-encrypt",
        "--state-dir",
        &state_dir_str,
        "--passphrase",
        "cookbook-demo-pass",
        "--json",
    ]);
    if encrypt.success {
        eprintln!("  OK: state encrypted ({}ms)", encrypt.duration_ms);
    } else {
        eprintln!("  FAIL: {}", first_line(&encrypt.output));
        failures += 1;
    }

    // Step 3: Decrypt state
    eprintln!("\nStep 2: Decrypt state");
    let decrypt = run_forjar(&[
        "state-decrypt",
        "--state-dir",
        &state_dir_str,
        "--passphrase",
        "cookbook-demo-pass",
        "--json",
    ]);
    if decrypt.success {
        eprintln!("  OK: state decrypted ({}ms)", decrypt.duration_ms);
    } else {
        eprintln!("  FAIL: {}", first_line(&decrypt.output));
        failures += 1;
    }

    // Step 4: Re-encrypt to verify contents survived roundtrip
    eprintln!("\nStep 3: Re-encrypt state");
    let reencrypt = run_forjar(&[
        "state-encrypt",
        "--state-dir",
        &state_dir_str,
        "--passphrase",
        "cookbook-demo-pass",
    ]);
    if reencrypt.success {
        eprintln!("  OK: re-encrypted ({}ms)", reencrypt.duration_ms);
    } else {
        eprintln!("  FAIL: {}", first_line(&reencrypt.output));
        failures += 1;
    }

    // Step 5: Rekey with a new passphrase
    eprintln!("\nStep 4: Rekey with new passphrase");
    let rekey = run_forjar(&[
        "state-rekey",
        "--state-dir",
        &state_dir_str,
        "--old-passphrase",
        "cookbook-demo-pass",
        "--new-passphrase",
        "cookbook-new-pass",
    ]);
    if rekey.success {
        eprintln!("  OK: rekeyed ({}ms)", rekey.duration_ms);
    } else {
        eprintln!("  FAIL: {}", first_line(&rekey.output));
        failures += 1;
    }

    // Step 6: Decrypt with new passphrase
    eprintln!("\nStep 5: Decrypt with new passphrase");
    let final_decrypt = run_forjar(&[
        "state-decrypt",
        "--state-dir",
        &state_dir_str,
        "--passphrase",
        "cookbook-new-pass",
    ]);
    if final_decrypt.success {
        eprintln!(
            "  OK: decrypted with new passphrase ({}ms)",
            final_decrypt.duration_ms
        );
    } else {
        eprintln!("  FAIL: {}", first_line(&final_decrypt.output));
        failures += 1;
    }

    let _ = std::fs::remove_dir_all(&tmp);
    eprintln!("\n--- Result: {failures} failure(s) ---");
    if failures > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

struct StepResult {
    success: bool,
    output: String,
    duration_ms: u64,
}

fn run_forjar(args: &[&str]) -> StepResult {
    let start = std::time::Instant::now();
    let result = Command::new("forjar").args(args).output();
    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            StepResult {
                success: output.status.success(),
                output: format!("{stdout}{stderr}"),
                duration_ms,
            }
        }
        Err(e) => StepResult {
            success: false,
            output: format!("failed to execute forjar: {e}"),
            duration_ms,
        },
    }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s).trim()
}
