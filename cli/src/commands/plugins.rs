//! `cowork plugins` command - Manage Claude Code marketplace plugins.

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

