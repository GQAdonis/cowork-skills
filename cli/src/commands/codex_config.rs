//! `codex_config` — idempotent Codex TOML config merge + goal template installer.
//!
//! Ported from prometheus-skill-pack `scripts/configure-mcp-all-tools.sh`
//! (merge_toml_mcp) and `scripts/kbd-goal-codex-setup.sh`.

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

/// MCP server definition for TOML injection.
struct McpStanza {
    /// TOML table key, e.g. "surreal-memory"
    key: &'static str,
    /// Lines to append for this stanza (without trailing newline on last line)
    lines: &'static str,
}

const MCP_STANZAS: &[McpStanza] = &[
    McpStanza {
        key: "surreal-memory",
        lines: "[mcp_servers.surreal-memory]\ntype = \"sse\"\nurl = \"http://localhost:23001/mcp/sse\"",
    },
    McpStanza {
        key: "prometheus-knowledge",
        lines: "[mcp_servers.prometheus-knowledge]\ntype = \"http\"\nurl = \"http://localhost:8942/mcp\"",
    },
    McpStanza {
        key: "forge-rs",
        lines: "[mcp_servers.forge-rs]\ntype = \"http\"\nurl = \"http://localhost:8943/mcp\"",
    },
    McpStanza {
        key: "sycophancy-correction",
        lines: "[mcp_servers.sycophancy-correction]\ncommand = \"/usr/local/bin/sycophancy-correction\"",
    },
    McpStanza {
        key: "sequential-thinking",
        lines: "[mcp_servers.sequential-thinking]\ncommand = \"npx\"\nargs = [\"-y\", \"@modelcontextprotocol/server-sequential-thinking\"]",
    },
];

/// Merge Prometheus MCP stanzas into a Codex config.toml file (idempotent).
///
/// Returns the list of stanza keys that were actually added.
/// Stanzas already present (detected by `[mcp_servers.<key>]` header) are skipped.
pub fn merge_codex_toml(config_path: &Path) -> Result<Vec<String>> {
    // Read existing content (or start empty if file doesn't exist yet)
    let existing = if config_path.exists() {
        fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?
    } else {
        String::new()
    };

    // Back up the original if it exists
    if config_path.exists() {
        let backup = config_path.with_extension("toml.bak");
        fs::copy(config_path, &backup)
            .with_context(|| format!("Failed to backup {}", config_path.display()))?;
    }

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut content = existing.clone();
    let mut added = Vec::new();

    for stanza in MCP_STANZAS {
        let header = format!("[mcp_servers.{}]", stanza.key);
        if !content.contains(&header) {
            // Append stanza with a leading blank line for readability
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            content.push('\n');
            content.push_str(stanza.lines);
            content.push('\n');
            added.push(stanza.key.to_string());
        }
    }

    fs::write(config_path, &content)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    Ok(added)
}

/// Set `goals.enabled = true` in a Codex config.toml (idempotent).
///
/// Returns `true` if the file was modified, `false` if it was already set.
pub fn set_goals_enabled(config_path: &Path) -> Result<bool> {
    let existing = if config_path.exists() {
        fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?
    } else {
        String::new()
    };

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Already set to true — nothing to do
    if existing.contains("goals.enabled = true") {
        return Ok(false);
    }

    let new_content = if existing.contains("goals.enabled") {
        // Replace any existing goals.enabled line
        existing
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("goals.enabled") {
                    "goals.enabled = true".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    } else {
        // Append the key
        let mut s = existing;
        if !s.ends_with('\n') && !s.is_empty() {
            s.push('\n');
        }
        s.push_str("goals.enabled = true\n");
        s
    };

    fs::write(config_path, &new_content)
        .with_context(|| format!("Failed to write {}", config_path.display()))?;

    Ok(true)
}

/// Copy KBD goal prompt templates to `~/.codex/goals/` (idempotent — overwrites).
///
/// `templates_src` should contain `continuation.md` and `budget_limit.md`.
/// Returns list of file names copied.
pub fn copy_goal_templates(templates_src: &Path, goals_dir: &Path) -> Result<Vec<String>> {
    let templates = ["continuation.md", "budget_limit.md"];
    let mut copied = Vec::new();

    fs::create_dir_all(goals_dir)
        .with_context(|| format!("Failed to create {}", goals_dir.display()))?;

    for name in &templates {
        let src = templates_src.join(name);
        if src.exists() {
            let dst = goals_dir.join(name);
            fs::copy(&src, &dst)
                .with_context(|| format!("Failed to copy {} to {}", src.display(), dst.display()))?;
            copied.push((*name).to_string());
        }
    }

    Ok(copied)
}

/// Locate the prometheus-skill-pack root by walking up from the cowork binary's
/// working directory, or accept an explicit override.
///
/// Looks for a directory containing `skills/process/kbd-goal/templates/codex/`.
fn find_pack_root(override_root: Option<&Path>) -> Option<PathBuf> {
    if let Some(root) = override_root {
        return Some(root.to_path_buf());
    }

    // Walk upward from CWD
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join("skills").join("process").join("kbd-goal").join("templates").join("codex");
        if candidate.is_dir() {
            return Some(dir);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => return None,
        }
    }
}

/// Top-level orchestrator: merge TOML config + set goals.enabled + copy templates.
///
/// `pack_root` is the prometheus-skill-pack root directory.
/// When `None`, auto-discovery is attempted.
pub fn configure_codex(pack_root: Option<&Path>) -> Result<()> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;
    let config_path = home.join(".codex").join("config.toml");

    println!("\n  {} Configuring Codex MCP + goals…", "→".blue());

    // ── 1. Merge MCP stanzas ──────────────────────────────────────────────
    let added = merge_codex_toml(&config_path)?;
    if added.is_empty() {
        println!("  {} config.toml MCP stanzas already complete", "✓".green());
    } else {
        println!("  {} Added MCP servers: {}", "✓".green(), added.join(", ").cyan());
    }

    // ── 2. Set goals.enabled ──────────────────────────────────────────────
    let modified = set_goals_enabled(&config_path)?;
    if modified {
        println!("  {} Set goals.enabled = true", "✓".green());
    } else {
        println!("  {} goals.enabled already true", "✓".green());
    }

    // ── 3. Copy goal templates ────────────────────────────────────────────
    let pack = find_pack_root(pack_root);
    match pack {
        Some(root) => {
            let templates_src = root
                .join("skills")
                .join("process")
                .join("kbd-goal")
                .join("templates")
                .join("codex");
            let goals_dir = home.join(".codex").join("goals");

            if templates_src.exists() {
                let copied = copy_goal_templates(&templates_src, &goals_dir)?;
                if copied.is_empty() {
                    println!("  {} No goal templates found at {}", "○".yellow(), templates_src.display());
                } else {
                    println!("  {} Copied goal templates: {}", "✓".green(), copied.join(", "));
                }
            } else {
                println!(
                    "  {} Goal templates not found ({}), skipping",
                    "○".yellow(),
                    templates_src.display()
                );
            }
        }
        None => {
            println!(
                "  {} prometheus-skill-pack root not found; goal templates not installed",
                "○".yellow()
            );
            println!("  {} Run `cowork config codex` from inside the skill-pack directory to install them", "→".blue());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_merge_codex_toml_fresh() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.toml");
        // Config doesn't exist yet
        let added = merge_codex_toml(&config).unwrap();
        assert!(added.contains(&"surreal-memory".to_string()));
        assert!(added.contains(&"prometheus-knowledge".to_string()));
        let content = fs::read_to_string(&config).unwrap();
        assert!(content.contains("[mcp_servers.surreal-memory]"));
        assert!(content.contains("[mcp_servers.sequential-thinking]"));
    }

    #[test]
    fn test_merge_codex_toml_idempotent() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.toml");
        // First pass: all stanzas added
        let added1 = merge_codex_toml(&config).unwrap();
        assert!(!added1.is_empty());
        // Second pass: nothing new
        let added2 = merge_codex_toml(&config).unwrap();
        assert!(added2.is_empty(), "second merge should add nothing");
    }

    #[test]
    fn test_merge_codex_toml_partial() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.toml");
        // Pre-existing partial config
        write_file(&config, "[mcp_servers.surreal-memory]\ntype = \"sse\"\nurl = \"http://localhost:23001/mcp/sse\"\n");
        let added = merge_codex_toml(&config).unwrap();
        // surreal-memory was already there
        assert!(!added.contains(&"surreal-memory".to_string()));
        // Others should have been added
        assert!(added.contains(&"prometheus-knowledge".to_string()));
    }

    #[test]
    fn test_set_goals_enabled_fresh() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.toml");
        let modified = set_goals_enabled(&config).unwrap();
        assert!(modified);
        let content = fs::read_to_string(&config).unwrap();
        assert!(content.contains("goals.enabled = true"));
    }

    #[test]
    fn test_set_goals_enabled_idempotent() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.toml");
        write_file(&config, "goals.enabled = true\n");
        let modified = set_goals_enabled(&config).unwrap();
        assert!(!modified, "should not modify when already set");
    }

    #[test]
    fn test_set_goals_enabled_replaces_false() {
        let tmp = TempDir::new().unwrap();
        let config = tmp.path().join("config.toml");
        write_file(&config, "goals.enabled = false\n");
        let modified = set_goals_enabled(&config).unwrap();
        assert!(modified);
        let content = fs::read_to_string(&config).unwrap();
        assert!(content.contains("goals.enabled = true"));
        assert!(!content.contains("goals.enabled = false"));
    }

    #[test]
    fn test_copy_goal_templates() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("templates");
        let dst = tmp.path().join("goals");
        fs::create_dir_all(&src).unwrap();
        write_file(&src.join("continuation.md"), "# continuation");
        write_file(&src.join("budget_limit.md"), "# budget");

        let copied = copy_goal_templates(&src, &dst).unwrap();
        assert_eq!(copied.len(), 2);
        assert!(dst.join("continuation.md").exists());
        assert!(dst.join("budget_limit.md").exists());
    }

    #[test]
    fn test_copy_goal_templates_missing_src() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("nonexistent");
        let dst = tmp.path().join("goals");
        // src doesn't exist — copy_goal_templates should return empty (no files found)
        let copied = copy_goal_templates(&src, &dst).unwrap();
        assert!(copied.is_empty());
    }
}
