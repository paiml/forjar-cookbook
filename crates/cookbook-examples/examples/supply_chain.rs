//! Supply chain security — SBOM, CBOM, provenance, and signing.
//!
//! Demonstrates `forjar sbom`, `forjar cbom`, `forjar provenance`,
//! `forjar sign`, and `forjar repro-proof` for supply chain attestation.
//!
//! Usage: `cargo run --example supply_chain`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-supply-chain");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Supply Chain Security ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r##"version: "1.0"
name: supply-chain-demo
machines:
  prod:
    hostname: prod-01
    addr: 10.0.1.5
    user: deploy
resources:
  runtime:
    type: package
    machine: prod
    provider: apt
    packages: [python3, python3-pip, nginx]
  app-binary:
    type: github_release
    machine: prod
    repo: myorg/myapp
    tag: v2.1.0
    asset_pattern: "*linux-amd64*"
    binary: myapp
    install_dir: /usr/local/bin
    depends_on: [runtime]
  tls-cert:
    type: file
    machine: prod
    path: /etc/ssl/certs/app.pem
    content: "# TLS certificate placeholder"
    owner: root
    mode: "0644"
    sudo: true
"##,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: SBOM (Software Bill of Materials)");
    let r = run_forjar(&["sbom", "-f", &f]);
    report("sbom", &r, &mut failures);

    eprintln!("Step 3: CBOM (Cryptographic Bill of Materials)");
    let r = run_forjar(&["cbom", "-f", &f]);
    report("cbom", &r, &mut failures);

    eprintln!("Step 4: SLSA Provenance");
    let r = run_forjar(&["provenance", "-f", &f]);
    report("provenance", &r, &mut failures);

    eprintln!("Step 5: Reproducibility proof");
    let r = run_forjar(&["repro-proof", "-f", &f]);
    report("repro-proof", &r, &mut failures);

    eprintln!("Step 6: Lineage (Merkle DAG)");
    let r = run_forjar(&["lineage", "-f", &f]);
    report("lineage", &r, &mut failures);

    eprintln!("Step 7: Sign help");
    let r = run_forjar(&["sign", "--help"]);
    report("sign-help", &r, &mut failures);

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
