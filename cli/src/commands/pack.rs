use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SKILL_DIRS: &[(&str, &str)] = &[
    ("claude-code", ".claude/skills"),
    ("kimi", ".kimi-code/skills"),
    ("minimax", ".minimax/skills"),
    ("opencode", ".opencode"),
    ("codex", ".codex"),
    ("cursor", ".cursor/skills"),
];

/// Resolve the prometheus-skill-pack root directory.
///
/// Resolution order:
/// 1. `PROMETHEUS_SKILL_PACK` environment variable
/// 2. `~/.cowork/prometheus-skill-pack/`
/// 3. `~/Projects/prometheus/prometheus-skill-pack` (local dev convenience)
pub fn resolve_pack_root() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("PROMETHEUS_SKILL_PACK") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let cowork_path = home.join(".cowork").join("prometheus-skill-pack");
        if cowork_path.exists() {
            return Some(cowork_path);
        }

        let dev_path = home
            .join("Projects")
            .join("prometheus")
            .join("prometheus-skill-pack");
        if dev_path.exists() {
            return Some(dev_path);
        }
    }

    None
}

/// Read the `version` field from `package.json` in the pack root.
fn read_pack_version(pack_root: &Path) -> Option<String> {
    let pkg = pack_root.join("package.json");
    let content = fs::read_to_string(&pkg).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("version")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Count how many `.md` skill files exist under `<home>/<rel_skills_dir>`.
fn count_installed_skills(skill_dir: &Path) -> usize {
    if !skill_dir.exists() {
        return 0;
    }
    fs::read_dir(skill_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let p = e.path();
                    p.is_dir()
                        || p.extension().and_then(|s| s.to_str()) == Some("md")
                })
                .count()
        })
        .unwrap_or(0)
}

/// Detect broken symlinks under `dir` (symlinks whose target does not exist).
pub fn find_broken_symlinks(dir: &Path) -> Vec<PathBuf> {
    let mut broken = Vec::new();
    if !dir.exists() {
        return broken;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_symlink() {
                match fs::metadata(&path) {
                    Ok(_) => {}
                    Err(_) => broken.push(path),
                }
            }
        }
    }
    broken
}

pub fn execute_status() -> Result<()> {
    let pack_root = resolve_pack_root().ok_or_else(|| {
        anyhow::anyhow!(
            "Prometheus skill-pack not found. Set PROMETHEUS_SKILL_PACK env var \
             or install to ~/.cowork/prometheus-skill-pack/"
        )
    })?;

    let version = read_pack_version(&pack_root)
        .unwrap_or_else(|| "unknown".to_string());

    println!("{}", "Prometheus Skill-Pack Status".bold());
    println!("  Root:    {}", pack_root.display());
    println!("  Version: {}", version.cyan());
    println!();

    let home = dirs::home_dir().unwrap_or_default();
    println!("{}", "Installed skills per platform:".bold());
    println!("  {:<14} {:<40} {}", "Platform", "Skills directory", "Count");
    println!("  {}", "-".repeat(62));

    for (platform, rel_dir) in SKILL_DIRS {
        let dir = home.join(rel_dir);
        let count = count_installed_skills(&dir);
        let status = if dir.exists() {
            format!("{}", count).green().to_string()
        } else {
            "—  (not installed)".dimmed().to_string()
        };
        println!("  {:<14} {:<40} {}", platform, rel_dir, status);
    }

    println!();
    println!(
        "Run {} to refresh all platforms.",
        "cowork pack update".yellow()
    );
    Ok(())
}

pub fn execute_update() -> Result<()> {
    let pack_root = resolve_pack_root().ok_or_else(|| {
        anyhow::anyhow!(
            "Prometheus skill-pack not found. Set PROMETHEUS_SKILL_PACK env var \
             or install to ~/.cowork/prometheus-skill-pack/"
        )
    })?;

    let script = pack_root.join("scripts").join("install-skills-flat.sh");
    if !script.exists() {
        anyhow::bail!(
            "Install script not found at {}",
            script.display()
        );
    }

    println!("{}", "Updating prometheus-skill-pack…".bold());
    println!("  Running: bash {}", script.display());

    let status = Command::new("bash")
        .arg(&script)
        .status()
        .with_context(|| format!("Failed to execute {}", script.display()))?;

    if status.success() {
        println!("{}", "  ✓ Update complete.".green());
    } else {
        anyhow::bail!("install-skills-flat.sh exited with status {}", status);
    }
    Ok(())
}

pub fn execute_repair() -> Result<()> {
    let pack_root = resolve_pack_root().ok_or_else(|| {
        anyhow::anyhow!(
            "Prometheus skill-pack not found. Set PROMETHEUS_SKILL_PACK env var \
             or install to ~/.cowork/prometheus-skill-pack/"
        )
    })?;

    let home = dirs::home_dir().unwrap_or_default();
    let mut any_broken = false;

    println!("{}", "Scanning for broken symlinks…".bold());

    for (platform, rel_dir) in SKILL_DIRS {
        let dir = home.join(rel_dir);
        let broken = find_broken_symlinks(&dir);
        if !broken.is_empty() {
            any_broken = true;
            println!(
                "  {} {} broken symlink(s) in {} ({})",
                "⚠".yellow(),
                broken.len(),
                platform,
                dir.display()
            );
            for b in &broken {
                println!("    • {}", b.display());
            }
        }
    }

    if !any_broken {
        println!("{}", "  ✓ No broken symlinks found.".green());
        return Ok(());
    }

    println!();
    println!("{}", "Repairing — running install-skills-flat.sh…".bold());

    let script = pack_root.join("scripts").join("install-skills-flat.sh");
    if !script.exists() {
        anyhow::bail!(
            "Install script not found at {}. \
             Run `cowork pack update` manually after fixing the script path.",
            script.display()
        );
    }

    let status = Command::new("bash")
        .arg(&script)
        .status()
        .with_context(|| format!("Failed to execute {}", script.display()))?;

    if status.success() {
        println!("{}", "  ✓ Repair complete.".green());
    } else {
        anyhow::bail!("install-skills-flat.sh exited with status {}", status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn resolve_pack_root_env_var() {
        let dir = TempDir::new().unwrap();
        // SAFETY: single-threaded test; no other threads read this env var concurrently.
        unsafe { std::env::set_var("PROMETHEUS_SKILL_PACK", dir.path()) };
        let result = resolve_pack_root();
        unsafe { std::env::remove_var("PROMETHEUS_SKILL_PACK") };
        assert_eq!(result.unwrap(), dir.path());
    }

    #[test]
    fn resolve_pack_root_missing_env_returns_none_or_dev() {
        unsafe { std::env::remove_var("PROMETHEUS_SKILL_PACK") };
        // Either finds the dev path or returns None — both are acceptable
        let _ = resolve_pack_root();
    }

    #[test]
    fn count_installed_skills_missing_dir_returns_zero() {
        let count = count_installed_skills(Path::new("/nonexistent/path/skills"));
        assert_eq!(count, 0);
    }

    #[test]
    fn count_installed_skills_counts_entries() {
        let dir = TempDir::new().unwrap();
        for name in &["a.md", "b.md", "c.md"] {
            fs::write(dir.path().join(name), "").unwrap();
        }
        let count = count_installed_skills(dir.path());
        assert_eq!(count, 3);
    }

    #[test]
    fn find_broken_symlinks_empty_for_missing_dir() {
        let broken = find_broken_symlinks(Path::new("/nonexistent/skills"));
        assert!(broken.is_empty());
    }

    #[test]
    fn find_broken_symlinks_detects_dangling() {
        let dir = TempDir::new().unwrap();
        let link = dir.path().join("dangling");
        // Create a symlink pointing at a non-existent target
        #[cfg(unix)]
        std::os::unix::fs::symlink("/nonexistent/target", &link).unwrap();
        #[cfg(not(unix))]
        {
            // On non-unix, skip this test gracefully
            return;
        }
        let broken = find_broken_symlinks(dir.path());
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0], link);
    }

    #[test]
    fn read_pack_version_from_package_json() {
        let dir = TempDir::new().unwrap();
        let mut f = fs::File::create(dir.path().join("package.json")).unwrap();
        write!(f, r#"{{"name":"prometheus-skill-pack","version":"1.5.3"}}"#).unwrap();
        let ver = super::read_pack_version(dir.path());
        assert_eq!(ver, Some("1.5.3".to_string()));
    }
}
