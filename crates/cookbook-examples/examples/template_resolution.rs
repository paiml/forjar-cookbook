//! Template resolution — parameter interpolation and recipe expansion.
//!
//! Demonstrates `forjar template`, `forjar show`, and `forjar explain`
//! for understanding how parameters resolve in configs and recipes.
//!
//! Usage: `cargo run --example template_resolution`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-template");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Template Resolution ---\n");

    // Config with params
    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: template-demo
params:
  app_port: "8080"
  app_name: "myapi"
  deploy_dir: "/opt/deploy"
machines:
  local:
    hostname: localhost
    addr: 127.0.0.1
    user: demo
resources:
  app-config:
    type: file
    machine: local
    path: "{{params.deploy_dir}}/{{params.app_name}}.conf"
    content: |
      name: {{params.app_name}}
      port: {{params.app_port}}
      host: {{machines.local.addr}}
  app-svc:
    type: service
    machine: local
    name: "{{params.app_name}}"
    enabled: true
    depends_on: [app-config]
"#,
    )
    .ok();

    // Recipe with inputs
    let recipe_path = tmp.join("recipe.yaml");
    std::fs::write(
        &recipe_path,
        r#"recipe:
  name: web-server
  version: "1.0"
  description: "Standard web server setup"
  inputs:
    port:
      type: string
      default: "80"
      description: "Listen port"
    workers:
      type: string
      default: "4"
      description: "Worker count"
resources:
  web-config:
    type: file
    machine: "{{inputs.machine}}"
    path: /etc/nginx/conf.d/default.conf
    content: |
      server {
        listen {{inputs.port}};
        worker_processes {{inputs.workers}};
      }
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate config with params");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Show resolved config");
    let r = run_forjar(&["show", "-f", &f]);
    report("show", &r, &mut failures);

    eprintln!("Step 3: Explain resource resolution");
    let r = run_forjar(&["explain", "-f", &f, "--resource", "app-config"]);
    report("explain", &r, &mut failures);

    eprintln!("Step 4: Template recipe expansion");
    let r = run_forjar(&[
        "template",
        &recipe_path.display().to_string(),
        "-V",
        "port=8443",
        "-V",
        "workers=8",
    ]);
    report("template", &r, &mut failures);

    eprintln!("Step 5: Schema export");
    let r = run_forjar(&["schema"]);
    report("schema", &r, &mut failures);

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
