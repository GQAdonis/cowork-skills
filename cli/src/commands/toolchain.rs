use anyhow::Result;
use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

use crate::commands::pack::resolve_pack_root;

/// A single entry from detect-toolchain.sh --json output.
#[derive(Debug, Deserialize)]
pub struct ToolEntry {
    pub status: String,
    #[serde(default)]
    pub version: String,
}

/// Full toolchain JSON map: key → ToolEntry.
pub type ToolchainMap = HashMap<String, ToolEntry>;

/// Groups of keys from detect-toolchain.sh, in display order.
const CORE_TOOLS: &[&str] = &["node", "npm", "git", "go", "docker"];
const RUST_TOOLS: &[&str] = &["rustc", "cargo", "rustup", "wasm32"];
const AI_CLIS: &[&str] = &["claude", "kimi", "mmx"];
const PROMETHEUS_BINS: &[&str] = &[
    "forge",
    "pk",
    "liter-llm",
    "prometheus",
    "prometheus-rust-auditor",
    "sycophancy-correction",
    "learner-model",
];
const MCP_SERVICES: &[&str] = &[
    "surreal-memory",
    "forge-rs",
    "prometheus-knowledge",
    "surface-bridge",
    "sovereign-sync-daemon",
];

/// Resolve the detect-toolchain.sh path from the pack root.
fn detect_script_path() -> Option<std::path::PathBuf> {
    resolve_pack_root().map(|root| {
        root.join("shared")
            .join("scripts")
            .join("detect-toolchain.sh")
    })
}

/// Run detect-toolchain.sh --json and return the parsed map.
/// Returns an error if the script is missing, fails, or produces non-JSON.
pub fn run_detect_toolchain() -> Result<ToolchainMap> {
    let script = detect_script_path().ok_or_else(|| {
        anyhow::anyhow!(
            "Prometheus skill-pack not found. Set PROMETHEUS_SKILL_PACK env var \
             or install to ~/.cowork/prometheus-skill-pack/"
        )
    })?;

    if !script.exists() {
        anyhow::bail!(
            "detect-toolchain.sh not found at {}. \
             Run `cowork pack update` to refresh the pack.",
            script.display()
        );
    }

    let output = Command::new("bash")
        .arg(&script)
        .arg("--json")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run detect-toolchain.sh: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let map: ToolchainMap = serde_json::from_str(stdout.trim()).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse detect-toolchain.sh JSON output: {}\nOutput was:\n{}",
            e,
            stdout
        )
    })?;
    Ok(map)
}

fn print_group(label: &str, keys: &[&str], map: &ToolchainMap) {
    println!("{}", label.bold());
    println!("  {:<32} {:<10} {}", "Tool", "Status", "Version");
    println!("  {}", "─".repeat(56));
    for key in keys {
        let entry = map.get(*key);
        let (status_str, version_str) = match entry {
            Some(e) => {
                let st = match e.status.as_str() {
                    "ok" => "✓ ok".green().to_string(),
                    "missing" => "✗ missing".red().to_string(),
                    "occupied" => "⚠ occupied".yellow().to_string(),
                    other => other.dimmed().to_string(),
                };
                (st, e.version.as_str().dimmed().to_string())
            }
            None => ("— n/a".dimmed().to_string(), String::new()),
        };
        println!("  {:<32} {:<18} {}", key, status_str, version_str);
    }
    println!();
}

pub fn execute_status() -> Result<()> {
    let map = run_detect_toolchain()?;

    println!("{}\n", "Prometheus Toolchain Status".bold());
    print_group("Core Tools", CORE_TOOLS, &map);
    print_group("Rust Toolchain", RUST_TOOLS, &map);
    print_group("AI Platform CLIs", AI_CLIS, &map);
    print_group("Prometheus Binaries", PROMETHEUS_BINS, &map);
    print_group("MCP Services", MCP_SERVICES, &map);

    let missing: Vec<&str> = PROMETHEUS_BINS
        .iter()
        .chain(MCP_SERVICES.iter())
        .chain(CORE_TOOLS.iter())
        .copied()
        .filter(|k| {
            map.get(*k)
                .map(|e| e.status != "ok")
                .unwrap_or(true)
        })
        .collect();

    if missing.is_empty() {
        println!("{}", "All required tools are healthy.".green());
    } else {
        println!(
            "{} {} item(s) missing or unhealthy. Run {} per item.",
            "⚠".yellow(),
            missing.len(),
            "cowork toolchain install <tool>".yellow()
        );
    }
    Ok(())
}

/// Returns true when all required binaries are healthy, false otherwise.
pub fn check_health(map: &ToolchainMap) -> bool {
    let required: &[&str] = &["node", "npm", "git", "rustc", "cargo"];
    required.iter().all(|k| {
        map.get(*k).map(|e| e.status == "ok").unwrap_or(false)
    })
}

pub fn execute_check() -> Result<()> {
    match run_detect_toolchain() {
        Ok(map) => {
            if check_health(&map) {
                println!("{}", "All required tools present.".green());
                Ok(())
            } else {
                eprintln!("{}", "One or more required tools are missing.".red());
                std::process::exit(1);
            }
        }
        Err(e) => {
            // Graceful degradation: script absent or pack not found
            eprintln!("{}: {}", "Toolchain check unavailable".yellow(), e);
            std::process::exit(1);
        }
    }
}

/// Per-tool install instructions.
fn install_instructions(tool: &str) -> Option<&'static str> {
    match tool {
        "node" | "npm" => Some(
            "Install Node.js: https://nodejs.org/\n  \
             or: brew install node  (macOS)\n  \
             or: sudo apt-get install nodejs npm  (Debian/Ubuntu)",
        ),
        "git" => Some(
            "Install Git: https://git-scm.com/downloads\n  \
             or: brew install git  (macOS)\n  \
             or: sudo apt-get install git  (Debian/Ubuntu)",
        ),
        "rustc" | "cargo" | "rustup" => Some(
            "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\n  \
             or: https://rustup.rs/",
        ),
        "go" => Some(
            "Install Go: https://go.dev/dl/\n  \
             or: brew install go  (macOS)",
        ),
        "docker" => Some(
            "Install Docker: https://docs.docker.com/get-docker/\n  \
             or: brew install --cask docker  (macOS)",
        ),
        "claude" => Some(
            "Install Claude Code: npm install -g @anthropic-ai/claude-code\n  \
             or: https://code.claude.com/",
        ),
        "kimi" => Some("Install Kimi Code CLI: https://kimi.ai/code"),
        "pk" => Some(
            "Build pk from source:\n  \
             cd ~/Projects/prometheus/prometheus-knowledge-rs && cargo build --release",
        ),
        "forge" => Some(
            "Build forge-rs:\n  \
             cd ~/Projects/prometheus/forge-rs && cargo build --release",
        ),
        "dsg" => Some(
            "Build dsg:\n  \
             cd ~/Projects/prometheus/disk-space-guardian && cargo build --release\n  \
             cp target/release/dsg ~/.local/bin/",
        ),
        "sycophancy-correction" => Some(
            "Build sycophancy-correction:\n  \
             cd skills/imported/sycophancy-correction && cargo build --release\n  \
             cp target/release/sycophancy-correction /usr/local/bin/",
        ),
        "surreal-memory" | "surreal-memory-native" => Some(
            "Rebuild and start the surreal-memory service:\n  \
             bash shared/scripts/detect-toolchain.sh  # check current status\n  \
             See docs/service-rebuild-runbook.md for full instructions.",
        ),
        "prometheus-knowledge" => Some(
            "Start the prometheus-knowledge MCP server:\n  \
             cd ~/Projects/prometheus/prometheus-knowledge-rs && cargo build --release\n  \
             See shared/scripts/detect-toolchain.sh for port details.",
        ),
        _ => None,
    }
}

pub fn execute_install(tool: &str) -> Result<()> {
    match install_instructions(tool) {
        Some(instructions) => {
            println!("{} {}", "Install instructions for".bold(), tool.cyan());
            println!();
            for line in instructions.lines() {
                println!("  {}", line);
            }
        }
        None => {
            println!(
                "{}: '{}' is not a known tool. Run {} to see all tools.",
                "Unknown tool".yellow(),
                tool,
                "cowork toolchain status".cyan()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(entries: &[(&str, &str, &str)]) -> ToolchainMap {
        entries
            .iter()
            .map(|(k, status, version)| {
                (
                    k.to_string(),
                    ToolEntry {
                        status: status.to_string(),
                        version: version.to_string(),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn check_health_all_ok() {
        let map = make_map(&[
            ("node", "ok", "20.0.0"),
            ("npm", "ok", "10.0.0"),
            ("git", "ok", "2.45.0"),
            ("rustc", "ok", "1.79.0"),
            ("cargo", "ok", "1.79.0"),
        ]);
        assert!(check_health(&map));
    }

    #[test]
    fn check_health_missing_tool() {
        let map = make_map(&[
            ("node", "ok", "20.0.0"),
            ("npm", "ok", "10.0.0"),
            ("git", "missing", ""),
            ("rustc", "ok", "1.79.0"),
            ("cargo", "ok", "1.79.0"),
        ]);
        assert!(!check_health(&map));
    }

    #[test]
    fn check_health_empty_map() {
        let map = make_map(&[]);
        assert!(!check_health(&map));
    }

    #[test]
    fn install_instructions_known_tool() {
        assert!(install_instructions("node").is_some());
        assert!(install_instructions("rustc").is_some());
        assert!(install_instructions("cargo").is_some());
        assert!(install_instructions("dsg").is_some());
    }

    #[test]
    fn install_instructions_unknown_tool() {
        assert!(install_instructions("totally-unknown-binary-xyz").is_none());
    }

    #[test]
    fn toolchain_json_parses_real_shape() {
        let json = r#"{
  "node": {"status": "ok", "version": "v20.11.0"},
  "npm": {"status": "ok", "version": "10.2.4"},
  "git": {"status": "missing", "version": ""},
  "rustc": {"status": "ok", "version": "1.79.0"},
  "cargo": {"status": "ok", "version": "1.79.0"}
}"#;
        let map: ToolchainMap = serde_json::from_str(json).unwrap();
        assert_eq!(map["node"].status, "ok");
        assert_eq!(map["git"].status, "missing");
        assert_eq!(map["rustc"].version, "1.79.0");
    }
}
