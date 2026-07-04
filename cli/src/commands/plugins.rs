//! `cowork plugins` command - Manage Claude Code marketplace plugins.

use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Plugin installation info from installed_plugins.json
#[derive(Debug, Clone, Deserialize, Serialize)]
struct PluginInstallation {
    scope: String,
    #[serde(rename = "installPath")]
    install_path: String,
    version: String,
    #[serde(rename = "installedAt")]
    installed_at: String,
    #[serde(rename = "lastUpdated")]
    last_updated: String,
    #[serde(rename = "gitCommitSha")]
    git_commit_sha: Option<String>,
    #[serde(rename = "projectPath")]
    project_path: Option<String>,
}

/// Installed plugins JSON structure
#[derive(Debug, Deserialize, Serialize)]
struct InstalledPlugins {
    version: u32,
    plugins: HashMap<String, Vec<PluginInstallation>>,
}

/// Settings JSON structure (partial)
#[derive(Debug, Deserialize, Serialize)]
struct ClaudeSettings {
    #[serde(rename = "enabledPlugins")]
    enabled_plugins: Option<HashMap<String, bool>>,
}

/// Claude Code plugin manifest (from .claude-plugin/plugin.json or plugin.json)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub skills: Vec<String>,
    pub license: String,
    /// Optional: human-readable description
    #[serde(default)]
    pub description: Option<String>,
}

/// Discover and validate a plugin manifest in a cloned repo directory.
/// Checks `.claude-plugin/plugin.json` first, then `plugin.json` at root.
pub fn validate_plugin_manifest(repo_dir: &Path) -> Result<PluginManifest> {
    let candidates = [
        repo_dir.join(".claude-plugin").join("plugin.json"),
        repo_dir.join("plugin.json"),
    ];

    let manifest_path = candidates
        .iter()
        .find(|p| p.exists())
        .context("No plugin.json found — expected at .claude-plugin/plugin.json or plugin.json")?;

    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

    let manifest: PluginManifest = serde_json::from_str(&content)
        .context("plugin.json is not valid JSON or is missing required fields (name, version, skills, license)")?;

    if manifest.name.is_empty() {
        anyhow::bail!("plugin.json: 'name' field is empty");
    }
    if manifest.version.is_empty() {
        anyhow::bail!("plugin.json: 'version' field is empty");
    }
    if manifest.skills.is_empty() {
        anyhow::bail!("plugin.json: 'skills' array is empty — plugin has no skills");
    }
    if manifest.license.is_empty() {
        anyhow::bail!("plugin.json: 'license' field is empty");
    }

    Ok(manifest)
}

/// Get Claude plugins directory
fn get_claude_plugins_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".claude").join("plugins"))
}

/// Shorten a path for display (show last 2-3 components)
fn shorten_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() <= 3 {
        path.to_string()
    } else {
        format!(".../{}", parts[parts.len()-2..].join("/"))
    }
}

/// Get Claude settings path
fn get_claude_settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".claude").join("settings.json"))
}

/// Load installed plugins
fn load_installed_plugins() -> Result<InstalledPlugins> {
    let plugins_dir = get_claude_plugins_dir()?;
    let installed_path = plugins_dir.join("installed_plugins.json");

    if !installed_path.exists() {
        return Ok(InstalledPlugins {
            version: 2,
            plugins: HashMap::new(),
        });
    }

    let content = fs::read_to_string(&installed_path)
        .context("Failed to read installed_plugins.json")?;

    serde_json::from_str(&content)
        .context("Failed to parse installed_plugins.json")
}

/// Load Claude settings
fn load_claude_settings() -> Result<ClaudeSettings> {
    let settings_path = get_claude_settings_path()?;

    if !settings_path.exists() {
        return Ok(ClaudeSettings {
            enabled_plugins: None,
        });
    }

    let content = fs::read_to_string(&settings_path)
        .context("Failed to read settings.json")?;

    serde_json::from_str(&content)
        .context("Failed to parse settings.json")
}

/// Execute the plugins list command
pub fn execute_list(verbose: bool) -> Result<()> {
    println!("{}\n", "Claude Code Marketplace Plugins".bold().underline());

    let installed = load_installed_plugins()?;
    let settings = load_claude_settings()?;
    let enabled_map = settings.enabled_plugins.unwrap_or_default();

    if installed.plugins.is_empty() {
        println!("  No marketplace plugins installed.");
        println!("\n  Install plugins with: {}", "/plugin install <name>".cyan());
        return Ok(());
    }

    // Group by scope
    let mut user_plugins: Vec<_> = Vec::new();
    let mut local_plugins: Vec<_> = Vec::new();

    for (plugin_id, installations) in &installed.plugins {
        for install in installations {
            let enabled = enabled_map.get(plugin_id).copied().unwrap_or(true);
            if install.scope == "user" {
                user_plugins.push((plugin_id, install, enabled));
            } else {
                local_plugins.push((plugin_id, install, enabled));
            }
        }
    }

    // Display user (global) plugins
    if !user_plugins.is_empty() {
        println!("{} ({})", "Global Plugins".cyan().bold(), user_plugins.len());
        for (plugin_id, install, enabled) in &user_plugins {
            let status = if *enabled {
                "✓".green()
            } else {
                "○".yellow()
            };
            let enabled_str = if *enabled { "" } else { " (disabled)" };

            println!(
                "  {} {}{}",
                status,
                plugin_id,
                enabled_str.dimmed()
            );

            if verbose {
                println!("      Version: {}", install.version);
                println!("      Path: {}", install.install_path.dimmed());
                if let Some(sha) = &install.git_commit_sha {
                    println!("      Commit: {}", &sha[..7.min(sha.len())]);
                }
                println!("      Installed: {}", install.installed_at);
            }
        }
    }

    // Display local (project) plugins
    if !local_plugins.is_empty() {
        if !user_plugins.is_empty() {
            println!();
        }
        println!("{} ({})", "Project Plugins".cyan().bold(), local_plugins.len());
        for (plugin_id, install, enabled) in &local_plugins {
            // For project-scoped plugins, the global enabledPlugins might not reflect
            // the actual state when in that project
            let status = "✓".green();
            let scope_info = if let Some(project) = &install.project_path {
                format!(" (in: {})", shorten_path(project))
            } else {
                String::new()
            };

            // Show global disabled state as a note, not as the primary status
            let global_note = if !*enabled {
                " [global: disabled]".dimmed().to_string()
            } else {
                String::new()
            };

            println!(
                "  {} {}{}{}",
                status,
                plugin_id,
                scope_info.dimmed(),
                global_note
            );

            if verbose {
                println!("      Version: {}", install.version);
                if let Some(project) = &install.project_path {
                    println!("      Project: {}", project);
                }
                println!("      Path: {}", install.install_path.dimmed());
                if let Some(sha) = &install.git_commit_sha {
                    println!("      Commit: {}", &sha[..7.min(sha.len())]);
                }
            }
        }
    }

    println!(
        "\n  Total: {} plugins",
        (user_plugins.len() + local_plugins.len()).to_string().green()
    );

    Ok(())
}

/// Execute the plugins status command
pub fn execute_status() -> Result<()> {
    println!("{}\n", "Plugin System Status".bold().underline());

    let project_root = std::env::current_dir()?;
    let project_claude_dir = project_root.join(".claude");

    // ====== Current Project Status ======
    println!("{}", "Current Project:".cyan().bold());
    println!("  {}", project_root.display());

    if project_claude_dir.exists() {
        // Check project plugins
        let mut project_plugins = Vec::new();
        let mut project_skills = Vec::new();

        for entry in fs::read_dir(&project_claude_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') || name == "settings.json" || name == "settings.local.json" {
                continue;
            }

            let path = entry.path();
            if path.is_dir() {
                // Check if it's a plugin (has .claude-plugin/)
                if path.join(".claude-plugin").exists() {
                    project_plugins.push(name);
                } else if name == "skills" {
                    // Check skills directory
                    for skill_entry in fs::read_dir(&path)? {
                        let skill_entry = skill_entry?;
                        let skill_name = skill_entry.file_name().to_string_lossy().to_string();
                        if !skill_name.starts_with('.') {
                            project_skills.push(skill_name);
                        }
                    }
                } else {
                    // Could be a plugin without .claude-plugin marker
                    project_plugins.push(format!("{} (unverified)", name));
                }
            }
        }

        // Show project plugins
        if !project_plugins.is_empty() {
            println!("\n  {}", "Installed Plugins:".cyan());
            for plugin in &project_plugins {
                println!("    {} {}", "✓".green(), plugin);
            }
        }

        // Show project skills
        if !project_skills.is_empty() {
            println!("\n  {}", "Installed Skills:".cyan());
            for skill in &project_skills {
                println!("    {} {}", "✓".green(), skill);
            }
        }

        // Check project settings.json
        let project_settings_path = project_claude_dir.join("settings.json");
        if project_settings_path.exists() {
            if let Ok(content) = fs::read_to_string(&project_settings_path) {
                if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(enabled) = settings.get("enabledPlugins") {
                        if let Some(obj) = enabled.as_object() {
                            println!("\n  {}", "Project Settings (enabledPlugins):".cyan());
                            for (id, val) in obj {
                                let status = if val.as_bool().unwrap_or(false) {
                                    "✓ enabled".green()
                                } else {
                                    "✗ disabled".red()
                                };
                                println!("    {} {}", status, id);
                            }
                        }
                    }
                }
            }
        }

        if project_plugins.is_empty() && project_skills.is_empty() {
            println!("\n  {} No plugins or skills installed in this project", "○".yellow());
        }
    } else {
        println!("\n  {} .claude/ directory not found", "○".yellow());
        println!("  {} Use 'cowork install <repo> --plugin --local' to install plugins", "→".blue());
    }

    // ====== Global Status ======
    println!("\n{}", "Global Status:".cyan().bold());

    let plugins_dir = get_claude_plugins_dir()?;
    let _settings_path = get_claude_settings_path()?;
    let _cache_dir = plugins_dir.join("cache");
    let marketplaces_dir = plugins_dir.join("marketplaces");

    println!(
        "  {} Plugins dir: {}",
        if plugins_dir.exists() { "✓".green() } else { "✗".red() },
        shorten_path(&plugins_dir.to_string_lossy())
    );
    println!(
        "  {} Marketplaces: {}",
        if marketplaces_dir.exists() { "✓".green() } else { "✗".red() },
        shorten_path(&marketplaces_dir.to_string_lossy())
    );

    // List registered marketplaces
    let known_path = plugins_dir.join("known_marketplaces.json");
    if known_path.exists() {
        if let Ok(content) = fs::read_to_string(&known_path) {
            if let Ok(known) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = known.as_object() {
                    println!("\n  {}", "Registered Marketplaces:".cyan());
                    for name in obj.keys() {
                        println!("    {} {}", "●".blue(), name);
                    }
                }
            }
        }
    }

    // Summary of all installed plugins
    let installed = load_installed_plugins()?;
    let settings = load_claude_settings()?;
    let enabled_map = settings.enabled_plugins.unwrap_or_default();

    let total = installed.plugins.len();
    let enabled = installed.plugins.keys()
        .filter(|id| enabled_map.get(*id).copied().unwrap_or(true))
        .count();
    let disabled = total - enabled;

    // Count project-specific plugins
    let project_path_str = project_root.to_string_lossy().to_string();
    let project_plugin_count = installed.plugins.values()
        .flat_map(|v| v.iter())
        .filter(|p| p.project_path.as_ref().map(|pp| pp == &project_path_str).unwrap_or(false))
        .count();

    println!("\n  {}", "Summary:".cyan());
    println!("    Total installed: {}", total.to_string().green());
    println!("    Enabled: {}", enabled.to_string().green());
    if disabled > 0 {
        println!("    Disabled: {}", disabled.to_string().yellow());
    }
    if project_plugin_count > 0 {
        println!("    For this project: {}", project_plugin_count.to_string().cyan());
    }

    Ok(())
}

/// Save installed plugins
fn save_installed_plugins(plugins: &InstalledPlugins) -> Result<()> {
    let plugins_dir = get_claude_plugins_dir()?;
    let installed_path = plugins_dir.join("installed_plugins.json");
    let content = serde_json::to_string_pretty(plugins)?;
    fs::write(&installed_path, content)?;
    Ok(())
}

/// Load full settings.json as Value for editing
fn load_settings_value() -> Result<serde_json::Value> {
    let settings_path = get_claude_settings_path()?;
    if !settings_path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = fs::read_to_string(&settings_path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Save settings.json
fn save_settings_value(value: &serde_json::Value) -> Result<()> {
    let settings_path = get_claude_settings_path()?;
    let content = serde_json::to_string_pretty(value)?;
    fs::write(&settings_path, content)?;
    Ok(())
}

/// Execute the plugins uninstall command - remove a marketplace plugin
pub fn execute_uninstall_plugin(plugin_id: &str) -> Result<()> {
    println!("{} Uninstalling marketplace plugin: {}\n", "→".blue(), plugin_id.cyan());

    let mut installed = load_installed_plugins()?;

    // Find the plugin (support partial match)
    let matched_id = installed.plugins.keys()
        .find(|k| k.as_str() == plugin_id || k.starts_with(&format!("{}@", plugin_id)))
        .cloned();

    let full_plugin_id = match matched_id {
        Some(id) => id,
        None => {
            // List available plugins
            println!("{} Plugin '{}' not found.\n", "✗".red(), plugin_id);
            println!("Available plugins:");
            for id in installed.plugins.keys() {
                println!("  - {}", id);
            }
            return Ok(());
        }
    };

    // Get install info before removing
    let installations = installed.plugins.get(&full_plugin_id).cloned();

    // 1. Remove from installed_plugins.json
    installed.plugins.remove(&full_plugin_id);
    save_installed_plugins(&installed)?;
    println!("  {} Removed from installed_plugins.json", "✓".green());

    // 2. Remove from settings.json enabledPlugins
    let mut settings = load_settings_value()?;
    if let Some(enabled) = settings.get_mut("enabledPlugins") {
        if let Some(obj) = enabled.as_object_mut() {
            if obj.remove(&full_plugin_id).is_some() {
                save_settings_value(&settings)?;
                println!("  {} Removed from settings.json", "✓".green());
            }
        }
    }

    // 3. Remove cache directory
    if let Some(installs) = installations {
        for install in installs {
            let cache_path = Path::new(&install.install_path);
            if cache_path.exists() {
                // Remove the version directory
                fs::remove_dir_all(cache_path)?;
                println!("  {} Removed cache: {}", "✓".green(), shorten_path(&install.install_path));

                // Also try to remove parent directory if empty (plugin name directory)
                if let Some(parent) = cache_path.parent() {
                    if parent.read_dir().map(|mut d| d.next().is_none()).unwrap_or(false) {
                        let _ = fs::remove_dir(parent);
                    }
                }
            }
        }
    }

    // 4. Remove plugin directory from ~/.claude/ if it exists (for --plugin installs)
    let plugin_name = full_plugin_id.split('@').next().unwrap_or(&full_plugin_id);
    let home = dirs::home_dir().context("Could not find home directory")?;
    let plugin_dir = home.join(".claude").join(plugin_name);
    if plugin_dir.exists() {
        if plugin_dir.is_symlink() {
            fs::remove_file(&plugin_dir)?;
        } else {
            fs::remove_dir_all(&plugin_dir)?;
        }
        println!("  {} Removed plugin directory: ~/.claude/{}", "✓".green(), plugin_name);
    }

    // 5. Remove skills symlinks from ~/.claude/skills/
    let skills_dir = home.join(".claude").join("skills");
    if skills_dir.exists() {
        let mut removed_skills = 0;
        for entry in fs::read_dir(&skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_symlink() {
                if let Ok(target) = fs::read_link(&path) {
                    let target_str = target.to_string_lossy();
                    if target_str.contains(plugin_name) {
                        fs::remove_file(&path)?;
                        removed_skills += 1;
                    }
                }
            }
        }
        if removed_skills > 0 {
            println!("  {} Removed {} skill symlinks", "✓".green(), removed_skills);
        }
    }

    println!("\n{} Plugin '{}' uninstalled successfully!", "✓".green(), full_plugin_id);
    println!("\n{} Restart Claude Code to apply changes", "ℹ".blue());

    Ok(())
}

/// Execute the plugins enable command
pub fn execute_enable_plugin(plugin_id: &str) -> Result<()> {
    let installed = load_installed_plugins()?;

    // Find the plugin
    let matched_id = installed.plugins.keys()
        .find(|k| k.as_str() == plugin_id || k.starts_with(&format!("{}@", plugin_id)))
        .cloned();

    let full_plugin_id = match matched_id {
        Some(id) => id,
        None => {
            println!("{} Plugin '{}' not found in installed plugins.", "✗".red(), plugin_id);
            return Ok(());
        }
    };

    // Update settings.json
    let mut settings = load_settings_value()?;

    if settings.get("enabledPlugins").is_none() {
        settings["enabledPlugins"] = serde_json::json!({});
    }

    settings["enabledPlugins"][&full_plugin_id] = serde_json::Value::Bool(true);
    save_settings_value(&settings)?;

    println!("{} Enabled plugin: {}", "✓".green(), full_plugin_id.cyan());
    println!("\n{} Restart Claude Code to apply changes", "ℹ".blue());

    Ok(())
}

/// Execute the plugins disable command
pub fn execute_disable_plugin(plugin_id: &str) -> Result<()> {
    let installed = load_installed_plugins()?;

    // Find the plugin
    let matched_id = installed.plugins.keys()
        .find(|k| k.as_str() == plugin_id || k.starts_with(&format!("{}@", plugin_id)))
        .cloned();

    let full_plugin_id = match matched_id {
        Some(id) => id,
        None => {
            println!("{} Plugin '{}' not found in installed plugins.", "✗".red(), plugin_id);
            return Ok(());
        }
    };

    // Update settings.json
    let mut settings = load_settings_value()?;

    if settings.get("enabledPlugins").is_none() {
        settings["enabledPlugins"] = serde_json::json!({});
    }

    settings["enabledPlugins"][&full_plugin_id] = serde_json::Value::Bool(false);
    save_settings_value(&settings)?;

    println!("{} Disabled plugin: {}", "✓".green(), full_plugin_id.cyan());
    println!("\n{} Restart Claude Code to apply changes", "ℹ".blue());

    Ok(())
}

/// List all marketplaces
pub fn execute_list_marketplaces() -> Result<()> {
    println!("{}\n", "Claude Code Marketplaces".bold().underline());

    let plugins_dir = get_claude_plugins_dir()?;
    let known_path = plugins_dir.join("known_marketplaces.json");

    if !known_path.exists() {
        println!("  No marketplaces configured.");
        return Ok(());
    }

    let content = fs::read_to_string(&known_path)?;
    let known: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = known.as_object() {
        for (name, info) in obj {
            let repo = info
                .get("source")
                .and_then(|s| s.get("repo"))
                .and_then(|r| r.as_str())
                .unwrap_or("unknown");

            let last_updated = info
                .get("lastUpdated")
                .and_then(|u| u.as_str())
                .map(|s| s.split('T').next().unwrap_or(s))
                .unwrap_or("unknown");

            println!("  {} {}", "●".cyan(), name.bold());
            println!("    Repo: {}", repo);
            println!("    Updated: {}", last_updated);

            // Count plugins in marketplace
            let marketplace_dir = plugins_dir.join("marketplaces").join(name);
            if marketplace_dir.exists() {
                let count = fs::read_dir(&marketplace_dir)
                    .map(|entries| {
                        entries
                            .filter_map(Result::ok)
                            .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('.'))
                            .count()
                    })
                    .unwrap_or(0);
                println!("    Plugins: {}", count);
            }
            println!();
        }

        println!("  Total: {} marketplaces", obj.len().to_string().green());
    }

    Ok(())
}

/// Remove a marketplace
pub fn execute_remove_marketplace(name: &str) -> Result<()> {
    println!("{} Removing marketplace: {}\n", "→".blue(), name.cyan());

    let plugins_dir = get_claude_plugins_dir()?;
    let known_path = plugins_dir.join("known_marketplaces.json");

    if !known_path.exists() {
        anyhow::bail!("No marketplaces configured.");
    }

    let content = fs::read_to_string(&known_path)?;
    let mut known: serde_json::Value = serde_json::from_str(&content)?;

    // Check if marketplace exists
    if known.get(name).is_none() {
        println!("{} Marketplace '{}' not found.\n", "✗".red(), name);
        println!("Available marketplaces:");
        if let Some(obj) = known.as_object() {
            for key in obj.keys() {
                println!("  - {}", key);
            }
        }
        return Ok(());
    }

    // 1. Remove from known_marketplaces.json
    if let Some(obj) = known.as_object_mut() {
        obj.remove(name);
    }
    let new_content = serde_json::to_string_pretty(&known)?;
    fs::write(&known_path, new_content)?;
    println!("  {} Removed from known_marketplaces.json", "✓".green());

    // 2. Remove marketplace directory
    let marketplace_dir = plugins_dir.join("marketplaces").join(name);
    if marketplace_dir.exists() {
        fs::remove_dir_all(&marketplace_dir)?;
        println!("  {} Removed marketplace directory", "✓".green());
    }

    println!("\n{} Marketplace '{}' removed successfully!", "✓".green(), name);
    println!("\n{} Restart Claude Code to apply changes", "ℹ".blue());

    Ok(())
}

/// Copy a directory tree recursively (src → dst, creating dst if absent).
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

/// Get the HEAD git commit SHA in a directory.
fn get_git_head_sha(dir: &Path) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Execute `cowork plugins install <git-url> [--local]`
///
/// Flow:
///   1. Clone git URL to a temp directory
///   2. Discover + validate `.claude-plugin/plugin.json` (or `plugin.json`)
///   3. Copy plugin to `~/.claude/<plugin-name>/` (user scope) or `.claude/<name>/` (local)
///   4. Register in `~/.claude/plugins/installed_plugins.json` (idempotent)
///   5. Add to `settings.json` `enabledPlugins` (idempotent)
///   6. Report installed skill paths
pub fn execute_install_plugin(git_url: &str, local: bool) -> Result<()> {
    println!("{} Installing Claude Code plugin from: {}\n", "→".blue(), git_url.cyan());

    // ── 1. Clone to temp dir ──────────────────────────────────────────────
    let tmp_dir = std::env::temp_dir().join(format!(
        "cowork-plugin-clone-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    ));

    println!("  {} Cloning repository…", "→".blue());
    let clone_status = Command::new("git")
        .args(["clone", "--depth=1", git_url, &tmp_dir.to_string_lossy()])
        .status()
        .context("Failed to run `git clone` — is git installed?")?;

    if !clone_status.success() {
        anyhow::bail!("git clone failed for URL: {}", git_url);
    }
    println!("  {} Cloned", "✓".green());

    // ── 2. Validate plugin.json ───────────────────────────────────────────
    let manifest = validate_plugin_manifest(&tmp_dir)?;
    println!(
        "  {} Validated plugin.json: {} v{} ({} skills)",
        "✓".green(),
        manifest.name.cyan(),
        manifest.version,
        manifest.skills.len()
    );

    // ── 3. Determine install path ─────────────────────────────────────────
    let install_base: PathBuf = if local {
        std::env::current_dir()
            .context("Cannot determine current directory")?
            .join(".claude")
            .join(&manifest.name)
    } else {
        dirs::home_dir()
            .context("Cannot find home directory")?
            .join(".claude")
            .join(&manifest.name)
    };

    // Copy the full repo into the install directory (idempotent overwrite).
    println!("  {} Installing to {}…", "→".blue(), install_base.display());
    copy_dir_all(&tmp_dir, &install_base)
        .with_context(|| format!("Failed to copy plugin to {}", install_base.display()))?;
    println!("  {} Installed", "✓".green());

    // ── 4. Get commit SHA ────────────────────────────────────────────────
    let commit_sha = get_git_head_sha(&tmp_dir);

    // ── 5. Register in installed_plugins.json ────────────────────────────
    let now = Utc::now().to_rfc3339();
    let scope = if local { "project" } else { "user" };
    let plugin_key = format!("{}@{}", manifest.name, manifest.name);

    let new_installation = PluginInstallation {
        scope: scope.to_string(),
        install_path: install_base.to_string_lossy().to_string(),
        version: manifest.version.clone(),
        installed_at: now.clone(),
        last_updated: now.clone(),
        git_commit_sha: commit_sha,
        project_path: if local {
            std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string())
        } else {
            None
        },
    };

    let mut installed = load_installed_plugins()?;
    let installs = installed.plugins.entry(plugin_key.clone()).or_default();

    // Idempotent: replace existing entry for this scope+path, or add new.
    let existing_idx = installs.iter().position(|i| i.scope == scope && i.install_path == new_installation.install_path);
    match existing_idx {
        Some(idx) => {
            installs[idx] = new_installation;
            println!("  {} Updated existing entry in installed_plugins.json", "✓".green());
        }
        None => {
            installs.push(new_installation);
            println!("  {} Registered in installed_plugins.json", "✓".green());
        }
    }

    let plugins_dir = get_claude_plugins_dir()?;
    fs::create_dir_all(&plugins_dir)?;
    save_installed_plugins(&installed)?;

    // ── 6. Update settings.json enabledPlugins ────────────────────────────
    let settings_path = if local {
        std::env::current_dir()
            .context("Cannot determine current directory")?
            .join(".claude")
            .join("settings.json")
    } else {
        get_claude_settings_path()?
    };

    let mut settings = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str::<serde_json::Value>(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if settings.get("enabledPlugins").is_none() {
        settings["enabledPlugins"] = serde_json::json!({});
    }
    // Only set to true when not already present (preserve explicit false).
    if settings["enabledPlugins"].get(&plugin_key).is_none() {
        settings["enabledPlugins"][&plugin_key] = serde_json::Value::Bool(true);
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_path, content)?;
        println!("  {} Registered in settings.json enabledPlugins", "✓".green());
    } else {
        println!("  {} enabledPlugins entry already present (no change)", "✓".green());
    }

    // ── 7. Cleanup temp dir ───────────────────────────────────────────────
    let _ = fs::remove_dir_all(&tmp_dir);

    // ── 8. Report installed skills ────────────────────────────────────────
    println!("\n{} Plugin '{}' installed successfully!\n", "✓".green(), manifest.name.cyan());
    println!("  {}", "Skills installed:".cyan());
    for skill in &manifest.skills {
        let skill_path = install_base.join(skill);
        println!("    {} {}", "●".blue(), skill_path.display());
    }
    println!("\n{} Restart Claude Code to activate the plugin", "ℹ".blue());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_plugin_json(dir: &Path, content: &str) {
        let claude_plugin_dir = dir.join(".claude-plugin");
        fs::create_dir_all(&claude_plugin_dir).unwrap();
        let mut f = fs::File::create(claude_plugin_dir.join("plugin.json")).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_validate_plugin_manifest_valid() {
        let tmp = TempDir::new().unwrap();
        write_plugin_json(
            tmp.path(),
            r#"{"name":"test-plugin","version":"1.0.0","skills":["skills/foo"],"license":"MIT"}"#,
        );
        let manifest = validate_plugin_manifest(tmp.path()).unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.skills, vec!["skills/foo"]);
        assert_eq!(manifest.license, "MIT");
    }

    #[test]
    fn test_validate_plugin_manifest_missing_name() {
        let tmp = TempDir::new().unwrap();
        write_plugin_json(
            tmp.path(),
            r#"{"name":"","version":"1.0.0","skills":["skills/foo"],"license":"MIT"}"#,
        );
        assert!(validate_plugin_manifest(tmp.path()).is_err());
    }

    #[test]
    fn test_validate_plugin_manifest_missing_skills() {
        let tmp = TempDir::new().unwrap();
        write_plugin_json(
            tmp.path(),
            r#"{"name":"test","version":"1.0.0","skills":[],"license":"MIT"}"#,
        );
        assert!(validate_plugin_manifest(tmp.path()).is_err());
    }

    #[test]
    fn test_validate_plugin_manifest_no_json() {
        let tmp = TempDir::new().unwrap();
        // No plugin.json anywhere — should fail
        assert!(validate_plugin_manifest(tmp.path()).is_err());
    }

    #[test]
    fn test_installed_plugins_idempotent_merge() {
        // Two installations of the same plugin key should result in 2 entries
        // (different scopes), but same scope+path should be replaced.
        let mut map: HashMap<String, Vec<PluginInstallation>> = HashMap::new();
        let entry = PluginInstallation {
            scope: "user".to_string(),
            install_path: "/home/user/.claude/myplugin".to_string(),
            version: "1.0.0".to_string(),
            installed_at: "2026-07-04T00:00:00Z".to_string(),
            last_updated: "2026-07-04T00:00:00Z".to_string(),
            git_commit_sha: Some("abc123".to_string()),
            project_path: None,
        };
        map.entry("myplugin@myplugin".to_string()).or_default().push(entry.clone());

        // Same scope+path → should replace
        let updated = PluginInstallation {
            version: "2.0.0".to_string(),
            git_commit_sha: Some("def456".to_string()),
            ..entry.clone()
        };
        let installs = map.get_mut("myplugin@myplugin").unwrap();
        let idx = installs.iter().position(|i| i.scope == "user" && i.install_path == "/home/user/.claude/myplugin");
        assert!(idx.is_some());
        installs[idx.unwrap()] = updated;

        let installs = map.get("myplugin@myplugin").unwrap();
        assert_eq!(installs.len(), 1, "idempotent: still only one entry");
        assert_eq!(installs[0].version, "2.0.0");
        assert_eq!(installs[0].git_commit_sha.as_deref(), Some("def456"));
    }
}
