//! Secret management — encrypted secrets with age provider.
//!
//! Demonstrates `forjar secrets` for managing age-encrypted secrets:
//! list, encrypt, and audit secret references in configs.
//!
//! Usage: `cargo run --example secret_management`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-secrets");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Secret Management ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: secrets-demo
machines:
  local:
    hostname: localhost
    addr: 127.0.0.1
    user: demo
secrets:
  provider: env
  ephemeral: true
resources:
  db-config:
    type: file
    machine: local
    path: /tmp/cookbook-secrets/db.conf
    content: "host=db.example.com user={{secrets.DB_USER}} pass={{secrets.DB_PASS}}"
  api-key:
    type: file
    machine: local
    path: /tmp/cookbook-secrets/api.env
    content: "API_KEY={{secrets.API_KEY}}"
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate config with secrets");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Secrets help (shows subcommands)");
    let r = run_forjar(&["secrets", "--help"]);
    report("secrets", &r, &mut failures);

    eprintln!("Step 3: Security scan");
    let r = run_forjar(&["security-scan", "-f", &f]);
    report("security-scan", &r, &mut failures);

    eprintln!("Step 4: Suggest improvements");
    let r = run_forjar(&["suggest", "-f", &f]);
    report("suggest", &r, &mut failures);

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
}

fn run_forjar(args: &[&str]) -> StepResult {
    match Command::new("forjar").args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            StepResult {
                success: output.status.success(),
                output: format!("{stdout}{stderr}"),
            }
        }
        Err(e) => StepResult {
            success: false,
            output: format!("failed to execute forjar: {e}"),
        },
    }
}

fn report(name: &str, r: &StepResult, failures: &mut u32) {
    if r.success {
        eprintln!("  {name}: OK");
        for line in r.output.lines().take(5) {
            if !line.trim().is_empty() {
                eprintln!("    {line}");
            }
        }
    } else {
        eprintln!(
            "  {name}: FAIL — {}",
            r.output.lines().next().unwrap_or("").trim()
        );
        *failures += 1;
    }
}
