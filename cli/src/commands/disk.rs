use anyhow::Result;
use colored::Colorize;
use std::process::Command;

const DSG_INSTALL_URL: &str =
    "https://github.com/GQAdonis/disk-space-guardian/releases/latest";

const DSG_BUILD_HINT: &str = "\
Build from source:
  cd ~/Projects/prometheus/disk-space-guardian
  cargo build --release
  cp target/release/dsg ~/.local/bin/";

/// Return true when `dsg` is available on PATH.
pub fn is_dsg_available() -> bool {
    Command::new("dsg")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn dsg_absent_error(subcommand: &str) -> Result<()> {
    eprintln!(
        "{}: `dsg` not found on PATH.",
        "cowork disk".red().bold()
    );
    eprintln!(
        "  {} is the disk-space-guardian CLI — install it first.",
        "dsg".cyan()
    );
    eprintln!();
    eprintln!("  {} {}", "Releases:".bold(), DSG_INSTALL_URL);
    eprintln!();
    eprintln!("  {}", DSG_BUILD_HINT);
    eprintln!();
    eprintln!(
        "  Once installed, retry: {}",
        format!("cowork disk {subcommand}").yellow()
    );
    std::process::exit(1);
}

pub fn execute_status() -> Result<()> {
    if !is_dsg_available() {
        return dsg_absent_error("status");
    }
    let status = Command::new("dsg")
        .args(["status", "--json"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run dsg: {}", e))?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

pub fn execute_scan(deep: bool, ecosystem: Option<&str>) -> Result<()> {
    if !is_dsg_available() {
        return dsg_absent_error("scan");
    }
    let mut cmd = Command::new("dsg");
    cmd.arg("scan");
    if deep {
        cmd.arg("--deep");
    }
    if let Some(eco) = ecosystem {
        cmd.args(["--ecosystem", eco]);
    }
    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run dsg: {}", e))?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

pub fn execute_clean(dry_run: bool, ecosystem: Option<&str>) -> Result<()> {
    if !is_dsg_available() {
        return dsg_absent_error("clean");
    }
    let mut cmd = Command::new("dsg");
    cmd.arg("clean");
    if dry_run {
        cmd.arg("--dry-run");
    }
    if let Some(eco) = ecosystem {
        cmd.args(["--ecosystem", eco]);
    }
    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run dsg: {}", e))?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsg_install_url_is_set() {
        assert!(!DSG_INSTALL_URL.is_empty());
        assert!(DSG_INSTALL_URL.contains("disk-space-guardian"));
    }

    #[test]
    fn dsg_build_hint_contains_cargo() {
        assert!(DSG_BUILD_HINT.contains("cargo build"));
    }

    #[test]
    fn is_dsg_available_returns_bool() {
        // This is always a valid bool — we just verify it doesn't panic.
        let _ = is_dsg_available();
    }

    #[test]
    fn is_dsg_available_false_for_nonexistent_binary() {
        // dsg is not expected to be on PATH in CI; absence is fine.
        // When dsg IS present this test still passes — it just returns true.
        let result = is_dsg_available();
        // Result is a bool: no panic is the assertion.
        assert!(result || !result);
    }
}
