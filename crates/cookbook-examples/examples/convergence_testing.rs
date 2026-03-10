//! Convergence testing — verify idempotency and convergence proofs.
//!
//! Demonstrates `forjar prove`, `forjar contracts`, `forjar fault-inject`,
//! and `forjar bench` for verifying infrastructure convergence properties.
//!
//! Usage: `cargo run --example convergence_testing`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-convergence");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Convergence Testing ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: convergence-demo
machines:
  local:
    hostname: localhost
    addr: 127.0.0.1
    user: demo
resources:
  base-pkgs:
    type: package
    machine: local
    provider: apt
    packages: [curl, jq]
  app-config:
    type: file
    machine: local
    path: /tmp/cookbook-conv/app.conf
    content: "port: 8080\nworkers: 4"
    depends_on: [base-pkgs]
  app-svc:
    type: service
    machine: local
    name: myapp
    enabled: true
    restart_on: [app-config]
    depends_on: [app-config]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Convergence proof");
    let r = run_forjar(&["prove", "-f", &f]);
    report("prove", &r, &mut failures);

    eprintln!("Step 2: Contract coverage");
    let r = run_forjar(&["contracts", "-f", &f]);
    report("contracts", &r, &mut failures);

    eprintln!("Step 3: Fault injection");
    let r = run_forjar(&["fault-inject", "-f", &f]);
    report("fault-inject", &r, &mut failures);

    eprintln!("Step 4: Preservation check");
    let r = run_forjar(&["preservation", "-f", &f]);
    report("preservation", &r, &mut failures);

    eprintln!("Step 5: Reproducibility certificate");
    let r = run_forjar(&["repro-proof", "-f", &f]);
    report("repro-proof", &r, &mut failures);

    eprintln!("Step 6: SLSA provenance");
    let r = run_forjar(&["provenance", "-f", &f]);
    report("provenance", &r, &mut failures);

    eprintln!("Step 7: Performance benchmarks");
    let r = run_forjar(&["bench", "--iterations", "100"]);
    report("bench", &r, &mut failures);

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
