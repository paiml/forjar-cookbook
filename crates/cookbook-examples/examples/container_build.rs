//! Container build — OCI image build and registry operations.
//!
//! Demonstrates `forjar build`, `forjar oci-pack`, and `forjar bundle`
//! for container image workflows.
//!
//! Usage: `cargo run --example container_build`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-container-build");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Container Build ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: container-build-demo
machines:
  build:
    hostname: build-server
    addr: 10.0.1.50
    user: ci
    container:
      runtime: docker
      image: ubuntu:22.04
      name: build-env
      ephemeral: true
resources:
  app-image:
    type: image
    machine: build
    image: myorg/webapp:latest
    command: "docker build -t myorg/webapp:latest ."
    working_dir: /home/ci/webapp
  base-tools:
    type: package
    machine: build
    provider: apt
    packages: [docker.io, curl]
"#,
    )
    .ok();

    // Create a directory to pack
    let pack_dir = tmp.join("pack-src");
    std::fs::create_dir_all(&pack_dir).ok();
    std::fs::write(pack_dir.join("app.py"), "print('hello')").ok();
    std::fs::write(pack_dir.join("requirements.txt"), "flask==3.0\n").ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Build help");
    let r = run_forjar(&["build", "--help"]);
    report("build-help", &r, &mut failures);

    eprintln!("Step 3: OCI pack");
    let r = run_forjar(&[
        "oci-pack",
        &pack_dir.display().to_string(),
        "--tag",
        "myorg/webapp:test",
        "--output",
        &tmp.join("oci-output").display().to_string(),
    ]);
    report("oci-pack", &r, &mut failures);

    eprintln!("Step 4: Bundle config");
    let r = run_forjar(&["bundle", "-f", &f]);
    report("bundle", &r, &mut failures);

    eprintln!("Step 5: SBOM");
    let r = run_forjar(&["sbom", "-f", &f]);
    report("sbom", &r, &mut failures);

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
