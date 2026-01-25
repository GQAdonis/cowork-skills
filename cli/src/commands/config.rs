//! `cowork config` command - Manage project-level skill configuration via Skills.toml.

use crate::skills_toml::{
    get_skill_groups, skills_lock_path, skills_toml_path, DevLink, DevLinkDetail, InstallScope,
    InstallType, LockedPackage, PackageSource, SkillDependency, SkillDependencyDetail, SkillsLock,
    SkillsToml, COWORK_DIR, SKILLS_TOML,
};
use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Detected package info for auto-detection
#[derive(Debug, Clone)]
pub struct DetectedPackage {
    pub name: String,
    pub path: String,
    pub is_plugin: bool,
    pub is_symlink: bool,
    pub scope: InstallScope,
    pub version: Option<String>,
}

/// Detect installed plugins and skills from global and project directories
fn detect_installed_packages(project_root: &Path) -> Vec<DetectedPackage> {
    let mut packages = Vec::new();
    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    // First, read from Claude Code's installed_plugins.json
    if let Some(home) = dirs::home_dir() {
        let installed_plugins_path = home.join(".claude").join("plugins").join("installed_plugins.json");
        if installed_plugins_path.exists() {
            if let Ok(content) = fs::read_to_string(&installed_plugins_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(plugins) = json.get("plugins").and_then(|p| p.as_object()) {
                        for (plugin_id, installs) in plugins {
                            // plugin_id format: "name@marketplace"
                            let name = plugin_id.split('@').next().unwrap_or(plugin_id).to_string();

                            if let Some(installs_array) = installs.as_array() {
                                for install in installs_array {
                                    let install_path = install.get("installPath")
                                        .and_then(|p| p.as_str())
                                        .unwrap_or("");
                                    let project_path = install.get("projectPath")
                                        .and_then(|p| p.as_str());
                                    let version = install.get("version")
                                        .and_then(|v| v.as_str())
                                        .map(String::from);
                                    let scope_str = install.get("scope")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("user");

                                    // Determine scope: local = project, user = global
                                    let scope = if scope_str == "local" {
                                        // Check if this project path matches current project
                                        if let Some(pp) = project_path {
                                            let pp_path = std::path::Path::new(pp);
                                            if pp_path == project_root {
                                                InstallScope::Project
                                            } else {
                                                // Skip plugins from other projects
                                                continue;
                                            }
                                        } else {
                                            continue;
                                        }
                                    } else {
                                        InstallScope::Global
                                    };

                                    // Skip if path doesn't exist
                                    let path = std::path::Path::new(install_path);
                                    if !path.exists() {
                                        continue;
                                    }

                                    // Skip if already seen
                                    if seen_paths.contains(install_path) {
                                        continue;
                                    }
                                    seen_paths.insert(install_path.to_string());

                                    let is_symlink = path.is_symlink();

                                    packages.push(DetectedPackage {
                                        name: name.clone(),
                                        path: install_path.to_string(),
                                        is_plugin: true,
                                        is_symlink,
                                        scope,
                                        version,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Also scan directories for plugins not in installed_plugins.json
    if let Some(home) = dirs::home_dir() {
        let global_claude = home.join(".claude");

        // Global plugins: ~/.claude/*/ (excluding skills/, plugins/, etc.)
        if global_claude.exists() {
            if let Ok(entries) = fs::read_dir(&global_claude) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        // Skip non-plugin directories
                        if ["skills", "plugins", "settings.json", "plans", "projects",
                            "cache", "commands", "hooks", "memory", "achievements",
                            "debug", "file-history", "paste-cache"].contains(&name.as_str())
                            || name.starts_with('.') {
                            continue;
                        }

                        let path_str = path.to_string_lossy().to_string();
                        if seen_paths.contains(&path_str) {
                            continue;
                        }

                        // Check if it's a valid plugin (has .claude-plugin or skills/)
                        let has_plugin_marker = path.join(".claude-plugin").exists();
                        let has_skills = path.join("skills").exists();
                        if has_plugin_marker || has_skills {
                            seen_paths.insert(path_str.clone());
                            let is_symlink = path.is_symlink();
                            let version = get_plugin_version_from_dir(&path);
                            packages.push(DetectedPackage {
                                name: name.clone(),
                                path: path_str,
                                is_plugin: true,
                                is_symlink,
                                scope: InstallScope::Global,
                                version,
                            });
                        }
                    }
                }
            }
        }

        // Global skills: ~/.claude/skills/
        let global_skills = global_claude.join("skills");
        if global_skills.exists() {
            if let Ok(entries) = fs::read_dir(&global_skills) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            continue;
                        }

                        let path_str = path.to_string_lossy().to_string();
                        if seen_paths.contains(&path_str) {
                            continue;
                        }

                        // Check if it has SKILL.md
                        if path.join("SKILL.md").exists() {
                            seen_paths.insert(path_str.clone());
                            let is_symlink = path.is_symlink();
                            packages.push(DetectedPackage {
                                name: name.clone(),
                                path: path_str,
                                is_plugin: false,
                                is_symlink,
                                scope: InstallScope::Global,
                                version: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Project .claude/
    let project_claude = project_root.join(".claude");

    // Project plugins: .claude/*/ (excluding skills/)
    if project_claude.exists() {
        if let Ok(entries) = fs::read_dir(&project_claude) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    // Skip non-plugin directories
                    if name == "skills" || name == "settings.json" || name.starts_with('.') {
                        continue;
                    }

                    let path_str = path.to_string_lossy().to_string();
                    if seen_paths.contains(&path_str) {
                        continue;
                    }

                    // Check if it's a valid plugin
                    let has_plugin_marker = path.join(".claude-plugin").exists();
                    let has_skills = path.join("skills").exists();
                    if has_plugin_marker || has_skills {
                        seen_paths.insert(path_str.clone());
                        let is_symlink = path.is_symlink();
                        let version = get_plugin_version_from_dir(&path);
                        packages.push(DetectedPackage {
                            name: name.clone(),
                            path: path_str,
                            is_plugin: true,
                            is_symlink,
                            scope: InstallScope::Project,
                            version,
                        });
                    }
                }
            }
        }
    }

    // Project skills: .claude/skills/
    let project_skills = project_claude.join("skills");
    if project_skills.exists() {
        if let Ok(entries) = fs::read_dir(&project_skills) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        continue;
                    }

                    let path_str = path.to_string_lossy().to_string();
                    if seen_paths.contains(&path_str) {
                        continue;
                    }

                    // Check if it has SKILL.md
                    if path.join("SKILL.md").exists() {
                        seen_paths.insert(path_str.clone());
                        let is_symlink = path.is_symlink();
                        packages.push(DetectedPackage {
                            name: name.clone(),
                            path: path_str,
                            is_plugin: false,
                            is_symlink,
                            scope: InstallScope::Project,
                            version: None,
                        });
                    }
                }
            }
        }
    }

    packages
}

/// Initialize skills.toml in the project
#[allow(dead_code)]
pub fn execute_init(project_root: &Path, force: bool) -> Result<()> {
    execute_init_with_options(project_root, force, true)
}

/// Initialize skills.toml with auto-detect option
pub fn execute_init_with_options(project_root: &Path, force: bool, auto_detect: bool) -> Result<()> {
    use std::io::{self, Write};

    let config_path = skills_toml_path(project_root);

    if config_path.exists() && !force {
        println!(
            "{} {}/{} already exists. Use --force to overwrite.",
            "!".yellow(),
            COWORK_DIR,
            SKILLS_TOML
        );
        return Ok(());
    }

    // Create .cowork directory if needed
    let cowork_dir = project_root.join(COWORK_DIR);
    if !cowork_dir.exists() {
        fs::create_dir_all(&cowork_dir)?;
    }

    // Get project name from directory
    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    // Auto-detect installed packages
    let detected = if auto_detect {
        detect_installed_packages(project_root)
    } else {
        Vec::new()
    };

    // If packages detected, ask user for confirmation
    let selected_packages = if !detected.is_empty() {
        println!("{}\n", "Detected installed plugins/skills:".bold());

        // Group by scope
        let global_plugins: Vec<_> = detected.iter().filter(|p| p.scope == InstallScope::Global && p.is_plugin).collect();
        let global_skills: Vec<_> = detected.iter().filter(|p| p.scope == InstallScope::Global && !p.is_plugin).collect();
        let project_plugins: Vec<_> = detected.iter().filter(|p| p.scope == InstallScope::Project && p.is_plugin).collect();
        let project_skills: Vec<_> = detected.iter().filter(|p| p.scope == InstallScope::Project && !p.is_plugin).collect();

        let mut idx = 1;
        let mut all_packages: Vec<&DetectedPackage> = Vec::new();

        if !global_plugins.is_empty() {
            println!("  {} (plugins):", "Global ~/.claude/".cyan());
            for pkg in &global_plugins {
                let symlink_mark = if pkg.is_symlink { " (symlink)" } else { "" };
                let version = pkg.version.as_deref().unwrap_or("");
                let version_str = if version.is_empty() { String::new() } else { format!(" v{}", version) };
                println!("    [{}] {}{}{}", idx, pkg.name.green(), version_str, symlink_mark.dimmed());
                all_packages.push(pkg);
                idx += 1;
            }
        }

        if !global_skills.is_empty() {
            println!("  {} (skills):", "Global ~/.claude/skills/".cyan());
            for pkg in &global_skills {
                let symlink_mark = if pkg.is_symlink { " (symlink)" } else { "" };
                println!("    [{}] {}{}", idx, pkg.name.green(), symlink_mark.dimmed());
                all_packages.push(pkg);
                idx += 1;
            }
        }

        if !project_plugins.is_empty() {
            println!("  {} (plugins):", "Project .claude/".cyan());
            for pkg in &project_plugins {
                let symlink_mark = if pkg.is_symlink { " (symlink)" } else { "" };
                let version = pkg.version.as_deref().unwrap_or("");
                let version_str = if version.is_empty() { String::new() } else { format!(" v{}", version) };
                println!("    [{}] {}{}{}", idx, pkg.name.green(), version_str, symlink_mark.dimmed());
                all_packages.push(pkg);
                idx += 1;
            }
        }

        if !project_skills.is_empty() {
            println!("  {} (skills):", "Project .claude/skills/".cyan());
            for pkg in &project_skills {
                let symlink_mark = if pkg.is_symlink { " (symlink)" } else { "" };
                println!("    [{}] {}{}", idx, pkg.name.green(), symlink_mark.dimmed());
                all_packages.push(pkg);
                idx += 1;
            }
        }

        println!();
        print!("{} Include in Skills.toml? [Y/n/select 1,2,3]: ", "?".blue());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            // Include all
            detected.clone()
        } else if input == "n" || input == "no" {
            // Include none
            Vec::new()
        } else {
            // Parse selection (e.g., "1,2,3" or "1-3")
            let mut selected = Vec::new();
            for part in input.split(',') {
                let part = part.trim();
                if part.contains('-') {
                    // Range like "1-3"
                    let range: Vec<&str> = part.split('-').collect();
                    if range.len() == 2 {
                        if let (Ok(start), Ok(end)) = (range[0].parse::<usize>(), range[1].parse::<usize>()) {
                            for i in start..=end {
                                if i >= 1 && i <= all_packages.len() {
                                    selected.push(all_packages[i - 1].clone());
                                }
                            }
                        }
                    }
                } else if let Ok(i) = part.parse::<usize>() {
                    if i >= 1 && i <= all_packages.len() {
                        selected.push(all_packages[i - 1].clone());
                    }
                }
            }
            selected
        }
    } else {
        Vec::new()
    };

    // Build config with selected packages
    let mut config = SkillsToml::default();
    config.project.name = Some(project_name.to_string());

    for pkg in &selected_packages {
        if pkg.is_symlink {
            // Add as dev link
            let dev_link = DevLink::Detailed(DevLinkDetail {
                path: get_symlink_target(&pkg.path).unwrap_or_else(|| pkg.path.clone()),
                local: pkg.scope == InstallScope::Project,
                enabled: true,
                plugin: pkg.is_plugin,
            });
            config.skills.dev.insert(pkg.name.clone(), dev_link);
        } else if pkg.is_plugin {
            // Add as installed dependency
            config.skills.install.insert(
                pkg.name.clone(),
                SkillDependency::Detailed(SkillDependencyDetail {
                    path: Some(pkg.path.clone()),
                    plugin: true,
                    local: pkg.scope == InstallScope::Project,
                    enabled: true,
                    ..Default::default()
                }),
            );
        } else {
            // Add skill to global enabled list
            config.skills.global.enabled.push(pkg.name.clone());
        }
    }

    // Save config
    config.save(&config_path)?;

    println!("{} Created {}", "✓".green(), config_path.display());

    if !selected_packages.is_empty() {
        println!("  {} packages configured", selected_packages.len());
    }

    println!("\n{}", "Next steps:".bold());
    println!(
        "  1. Edit {}/{} to configure your skills",
        COWORK_DIR, SKILLS_TOML
    );
    println!("  2. Run 'cowork config show' to verify");
    println!("  3. Run 'cowork config install' to install dependencies");

    Ok(())
}

/// Get symlink target path
fn get_symlink_target(path: &str) -> Option<String> {
    fs::read_link(path).ok().map(|p| p.to_string_lossy().to_string())
}

/// Show current skills.toml configuration
pub fn execute_show(project_root: &Path) -> Result<()> {
    let config = load_config(project_root)?;

    println!("{}\n", "Skills Configuration".bold().underline());

    // Project info
    if let Some(name) = &config.project.name {
        println!("{}: {}", "Project".cyan(), name);
    }
    if let Some(desc) = &config.project.description {
        println!("{}: {}", "Description".cyan(), desc);
    }
    println!();

    // Global skills
    println!("{}", "Global Skills (~/.claude/skills/)".bold());
    if !config.skills.global.enabled.is_empty() {
        println!("  {}: {}", "Enabled".green(), config.skills.global.enabled.join(", "));
    }
    if !config.skills.global.disabled.is_empty() {
        println!("  {}: {}", "Disabled".red(), config.skills.global.disabled.join(", "));
    }
    if config.skills.global.enabled.is_empty() && config.skills.global.disabled.is_empty() {
        println!("  (all skills enabled)");
    }
    println!();

    // Skill groups
    println!("{}", "Skill Groups".bold());
    if !config.skills.groups.enabled.is_empty() {
        println!("  {}: {}", "Enabled".green(), config.skills.groups.enabled.join(", "));
    }
    if !config.skills.groups.disabled.is_empty() {
        println!("  {}: {}", "Disabled".red(), config.skills.groups.disabled.join(", "));
    }
    if config.skills.groups.enabled.is_empty() && config.skills.groups.disabled.is_empty() {
        println!("  (no groups configured)");
    }
    println!();

    // Dependencies
    if !config.skills.install.is_empty() {
        println!("{}", "Dependencies".bold());
        for (name, dep) in &config.skills.install {
            match dep {
                SkillDependency::Simple(repo) => {
                    println!("  {} = \"{}\" (global)", name.cyan(), repo);
                }
                SkillDependency::Detailed(d) => {
                    let source = d.repo.as_deref()
                        .or(d.path.as_deref())
                        .unwrap_or("?");
                    let mut extras = Vec::new();
                    if d.plugin {
                        extras.push("plugin".to_string());
                    }
                    if d.local {
                        extras.push("local".to_string());
                    } else {
                        extras.push("global".to_string());
                    }
                    if !d.enabled {
                        extras.push("disabled".to_string());
                    }
                    if !d.skills.is_empty() {
                        extras.push(format!("skills: [{}]", d.skills.join(", ")));
                    }
                    if let Some(r) = &d.git_ref {
                        extras.push(format!("ref: {}", r));
                    }
                    let name_display = if d.enabled {
                        name.cyan()
                    } else {
                        name.dimmed()
                    };
                    println!("  {} = \"{}\" ({})", name_display, source, extras.join(", "));
                }
            }
        }
        println!();
    }

    // Dev Links
    if !config.skills.dev.is_empty() {
        println!("{}", "Dev Links".bold());
        for (name, link) in &config.skills.dev {
            let target = if link.is_local() { "local" } else { "global" };
            let enabled_str = if link.is_enabled() { "" } else { ", disabled" };
            let name_display = if link.is_enabled() {
                name.cyan()
            } else {
                name.dimmed()
            };
            println!("  {} -> \"{}\" ({}{})", name_display, link.path(), target, enabled_str);
        }
        println!();
    }

    // Triggers
    if !config.triggers.priority.is_empty() || !config.triggers.overrides.is_empty() {
        println!("{}", "Triggers".bold());
        if !config.triggers.priority.is_empty() {
            println!("  {}: {}", "Priority".cyan(), config.triggers.priority.join(" > "));
        }
        if !config.triggers.overrides.is_empty() {
            println!("  {}:", "Overrides".cyan());
            for (trigger, skill) in &config.triggers.overrides {
                println!("    \"{}\" -> {}", trigger, skill);
            }
        }
    }

    Ok(())
}

/// Enable skills or groups
pub fn execute_enable(project_root: &Path, names: &[String]) -> Result<()> {
    let mut config = load_config(project_root)?;
    let groups = get_skill_groups();

    for name in names {
        if groups.contains_key(name.as_str()) {
            // It's a group
            if !config.skills.groups.enabled.contains(name) {
                config.skills.groups.enabled.push(name.clone());
            }
            config.skills.groups.disabled.retain(|n| n != name);
            println!("{} Enabled group: {}", "✓".green(), name.cyan());
        } else {
            // It's a skill - add to global enabled
            if !config.skills.global.enabled.contains(name) {
                config.skills.global.enabled.push(name.clone());
            }
            config.skills.global.disabled.retain(|n| n != name);
            println!("{} Enabled skill: {}", "✓".green(), name.cyan());
        }
    }

    save_config(project_root, &config)?;
    Ok(())
}

/// Disable skills or groups
pub fn execute_disable(project_root: &Path, names: &[String]) -> Result<()> {
    let mut config = load_config(project_root)?;
    let groups = get_skill_groups();

    for name in names {
        if groups.contains_key(name.as_str()) {
            // It's a group
            if !config.skills.groups.disabled.contains(name) {
                config.skills.groups.disabled.push(name.clone());
            }
            config.skills.groups.enabled.retain(|n| n != name);
            println!("{} Disabled group: {}", "✓".green(), name.cyan());
        } else {
            // It's a skill - add to global disabled
            if !config.skills.global.disabled.contains(name) {
                config.skills.global.disabled.push(name.clone());
            }
            config.skills.global.enabled.retain(|n| n != name);
            println!("{} Disabled skill: {}", "✓".green(), name.cyan());
        }
    }

    save_config(project_root, &config)?;
    Ok(())
}

/// Set skill priority for trigger conflicts
pub fn execute_priority(project_root: &Path, skills: &[String]) -> Result<()> {
    let mut config = load_config(project_root)?;

    config.triggers.priority = skills.to_vec();
    save_config(project_root, &config)?;

    println!("{} Set trigger priority:", "✓".green());
    for (i, skill) in skills.iter().enumerate() {
        println!("  {}. {}", i + 1, skill.cyan());
    }

    Ok(())
}

/// Override a specific trigger
pub fn execute_override(project_root: &Path, trigger: &str, skill: &str) -> Result<()> {
    let mut config = load_config(project_root)?;

    config.triggers.overrides.insert(trigger.to_string(), skill.to_string());
    save_config(project_root, &config)?;

    println!(
        "{} Override: \"{}\" -> {}",
        "✓".green(),
        trigger.yellow(),
        skill.cyan()
    );

    Ok(())
}

/// Add a dependency to skills.toml
pub fn execute_add(project_root: &Path, name: &str, source: &str, options: AddOptions) -> Result<()> {
    let mut config = load_config(project_root)?;

    // Handle dev link (symlink for testing)
    // Note: dev links default to local (project-level) since you're testing in your project
    if options.dev {
        // For dev links: default is local=true, use --local=false (not supported yet) for global
        // Use simple form if local and enabled, detailed form otherwise
        let is_local = true; // Dev links always local for now
        let dev_link = if !options.disabled && !options.plugin {
            DevLink::Simple(source.to_string())
        } else {
            DevLink::Detailed(DevLinkDetail {
                path: source.to_string(),
                local: is_local,
                enabled: !options.disabled,
                plugin: options.plugin,
            })
        };

        config.skills.dev.insert(name.to_string(), dev_link);
        save_config(project_root, &config)?;

        let mut status_parts = Vec::new();
        if options.plugin {
            status_parts.push("plugin");
        }
        status_parts.push("local");
        if options.disabled {
            status_parts.push("disabled");
        }
        let status = status_parts.join(", ");
        println!(
            "{} Added dev link: {} -> \"{}\" ({})",
            "✓".green(),
            name.cyan(),
            source,
            status
        );
        return Ok(());
    }

    // Handle regular dependency
    let needs_detailed = !options.skills.is_empty()
        || options.git_ref.is_some()
        || !options.agents.is_empty()
        || options.plugin
        || options.local
        || options.disabled;

    let dep = if !needs_detailed {
        // Simple form (global, enabled)
        if source.starts_with("../") || source.starts_with("./") || source.starts_with('/') {
            // Local path always needs detailed form
            SkillDependency::Detailed(SkillDependencyDetail {
                path: Some(source.to_string()),
                ..Default::default()
            })
        } else {
            SkillDependency::Simple(source.to_string())
        }
    } else {
        // Detailed form
        let (repo, path) = if source.starts_with("../") || source.starts_with("./") || source.starts_with('/') {
            (None, Some(source.to_string()))
        } else {
            (Some(source.to_string()), None)
        };

        SkillDependency::Detailed(SkillDependencyDetail {
            repo,
            path,
            skills: options.skills,
            git_ref: options.git_ref,
            agents: options.agents,
            plugin: options.plugin,
            local: options.local,
            enabled: !options.disabled,
        })
    };

    config.skills.install.insert(name.to_string(), dep);
    save_config(project_root, &config)?;

    // Build description
    let mut desc_parts = Vec::new();
    if options.plugin {
        desc_parts.push("plugin");
    }
    if options.local {
        desc_parts.push("local");
    }
    if options.disabled {
        desc_parts.push("disabled");
    }

    let dep_type = if desc_parts.is_empty() {
        "dependency".to_string()
    } else {
        desc_parts.join(", ")
    };

    println!(
        "{} Added {}: {} = \"{}\"",
        "✓".green(),
        dep_type,
        name.cyan(),
        source
    );
    println!("\n{} Run 'cowork config install' to install", "→".blue());

    Ok(())
}

/// Options for adding a dependency
#[derive(Default)]
pub struct AddOptions {
    pub skills: Vec<String>,
    pub git_ref: Option<String>,
    pub agents: Vec<String>,
    pub plugin: bool,
    pub local: bool,
    pub disabled: bool,
    pub dev: bool,
}

/// Remove a dependency from skills.toml
pub fn execute_remove(project_root: &Path, name: &str) -> Result<()> {
    let mut config = load_config(project_root)?;

    if config.skills.install.remove(name).is_some() {
        save_config(project_root, &config)?;
        println!("{} Removed dependency: {}", "✓".green(), name.cyan());
    } else {
        println!("{} Dependency not found: {}", "!".yellow(), name);
    }

    Ok(())
}

/// Install dependencies from Skills.toml
pub fn execute_install_deps(project_root: &Path) -> Result<()> {
    let config = load_config(project_root)?;

    let has_deps = !config.skills.install.is_empty();
    let has_dev = !config.skills.dev.is_empty();

    if !has_deps && !has_dev {
        println!("{} No dependencies to install", "ℹ".blue());
        return Ok(());
    }

    // Load or create lock file
    let mut lock = SkillsLock::load_from_project(project_root)?
        .unwrap_or_default();

    let mut installed_count = 0;
    let mut failed_count = 0;

    // Install regular dependencies
    if has_deps {
        println!("{}\n", "Installing dependencies from Skills.toml".bold());

        for (name, dep) in &config.skills.install {
            // Skip disabled dependencies
            if !dep.is_enabled() {
                println!("{} {} (disabled, skipping)", "○".dimmed(), name.dimmed());
                continue;
            }

            let mut desc_parts = Vec::new();
            if dep.is_plugin() {
                desc_parts.push("plugin");
            }
            if dep.is_local() {
                desc_parts.push("local");
            } else {
                desc_parts.push("global");
            }
            let dep_type = desc_parts.join(", ");

            println!("{} {} ({})...", "→".blue(), name.cyan(), dep_type);

            let result = install_from_dependency(name, dep, project_root);

            match result {
                Ok(locked_pkg) => {
                    println!("  {} Installed", "✓".green());
                    lock.upsert_package(locked_pkg);
                    installed_count += 1;
                }
                Err(e) => {
                    println!("  {} Failed: {}", "✗".red(), e);
                    failed_count += 1;
                }
            }
        }
    }

    // Install dev links
    if has_dev {
        println!("\n{}\n", "Creating dev links".bold());

        for (name, link) in &config.skills.dev {
            if !link.is_enabled() {
                println!("{} {} (disabled, skipping)", "○".dimmed(), name.dimmed());
                continue;
            }

            let mut desc_parts = Vec::new();
            if link.is_plugin() {
                desc_parts.push("plugin");
            }
            desc_parts.push(if link.is_local() { "local" } else { "global" });
            let link_type = desc_parts.join(", ");
            println!("{} {} -> {} ({})...", "→".blue(), name.cyan(), link.path(), link_type);

            let result = create_dev_link(name, link, project_root);

            match result {
                Ok(locked_pkg) => {
                    println!("  {} Linked", "✓".green());
                    lock.upsert_package(locked_pkg);
                    installed_count += 1;
                }
                Err(e) => {
                    println!("  {} Failed: {}", "✗".red(), e);
                    failed_count += 1;
                }
            }
        }
    }

    // Save lock file
    if installed_count > 0 {
        let lock_path = lock.save_to_project(project_root)?;
        println!("\n{} Updated {}", "✓".green(), lock_path.display());
    }

    if failed_count > 0 {
        println!("\n{} {} installed, {} failed", "Summary:".bold(), installed_count, failed_count);
    }

    Ok(())
}

/// Install from a SkillDependency and return lock info
fn install_from_dependency(name: &str, dep: &SkillDependency, project_root: &Path) -> Result<LockedPackage> {
    use crate::commands::install::{execute, get_install_info, InstallOptions};

    let source = dep.repo().or(dep.path()).ok_or_else(|| {
        anyhow::anyhow!("No repo or path specified")
    })?;

    let options = InstallOptions {
        repo: Some(source.to_string()),
        skills: dep.skills().to_vec(),
        agents: dep.agents().to_vec(),
        plugin: dep.is_plugin(),
        local: dep.is_local(),
        yes: true, // Auto-confirm for config install
        ..Default::default()
    };

    execute(options.clone())?;

    // Get installation info for lock file
    let install_info = get_install_info(&options, project_root)?;

    // Build locked package
    let is_github = dep.repo().is_some();
    let mut locked = if is_github {
        LockedPackage::from_github(
            name,
            source,
            install_info.version,
            install_info.git_sha,
            dep.git_ref().map(String::from),
        )
    } else {
        let mut pkg = LockedPackage::new_with_timestamp(name);
        pkg.source = PackageSource::Local;
        pkg.source_path = Some(source.to_string());
        if let Some(v) = install_info.version {
            pkg.version = v;
        }
        pkg
    };

    // Set install type and scope
    if dep.is_plugin() {
        locked.install_type = InstallType::Plugin;
    }
    locked.scope = if dep.is_local() {
        InstallScope::Project
    } else {
        InstallScope::Global
    };
    locked.install_path = install_info.install_path;
    locked.skills = install_info.skills;

    Ok(locked)
}

/// Create a development symlink and return lock info
fn create_dev_link(name: &str, link: &DevLink, _project_root: &Path) -> Result<LockedPackage> {
    use std::os::unix::fs::symlink;

    let source_path = std::path::Path::new(link.path());
    if !source_path.exists() {
        return Err(anyhow::anyhow!("Source path does not exist: {}", link.path()));
    }

    let base_dir = if link.is_local() {
        // Project .claude/
        std::env::current_dir()?.join(".claude")
    } else {
        // Global ~/.claude/
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".claude")
    };

    let link_path = if link.is_plugin() {
        // Plugin: link directly to .claude/<name>/
        base_dir.join(name)
    } else {
        // Skill: link to .claude/skills/<name>
        let skills_dir = base_dir.join("skills");
        if !skills_dir.exists() {
            fs::create_dir_all(&skills_dir)?;
        }
        skills_dir.join(name)
    };

    // Create parent directory if needed
    if let Some(parent) = link_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Remove existing link if any
    if link_path.exists() || link_path.is_symlink() {
        fs::remove_file(&link_path).or_else(|_| fs::remove_dir_all(&link_path))?;
    }

    // Create symlink
    symlink(source_path, &link_path)?;

    // Return lock info
    let mut locked = LockedPackage::from_dev_link(
        name,
        link.path(),
        &link_path.to_string_lossy(),
        link.is_local(),
    );
    if link.is_plugin() {
        locked.install_type = InstallType::Plugin;
    }
    Ok(locked)
}

/// Apply configuration (generate SKILLS.md or similar)
pub fn execute_apply(project_root: &Path, output: Option<&Path>) -> Result<()> {
    let config = load_config(project_root)?;
    let groups = get_skill_groups();

    let mut enabled_skills: Vec<String> = Vec::new();

    // Add skills from enabled groups
    for group_name in &config.skills.groups.enabled {
        if let Some(skills) = groups.get(group_name.as_str()) {
            for skill in skills {
                if !enabled_skills.contains(&skill.to_string()) {
                    enabled_skills.push(skill.to_string());
                }
            }
        }
    }

    // Add explicitly enabled skills
    for skill in &config.skills.global.enabled {
        if !enabled_skills.contains(skill) {
            enabled_skills.push(skill.clone());
        }
    }

    // Remove disabled skills
    for skill in &config.skills.global.disabled {
        enabled_skills.retain(|s| s != skill);
    }

    // Remove skills from disabled groups
    for group_name in &config.skills.groups.disabled {
        if let Some(skills) = groups.get(group_name.as_str()) {
            for skill in skills {
                enabled_skills.retain(|s| s != *skill);
            }
        }
    }

    // Generate output
    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| project_root.join("SKILLS.md"));

    let content = generate_skills_md(&config, &enabled_skills);
    fs::write(&output_path, content)?;

    println!("{} Generated {}", "✓".green(), output_path.display());
    println!("  {} skills configured", enabled_skills.len());

    Ok(())
}

fn generate_skills_md(config: &SkillsToml, enabled_skills: &[String]) -> String {
    let mut content = String::new();

    content.push_str("# Project Skills Configuration\n\n");

    if let Some(name) = &config.project.name {
        content.push_str(&format!("> Generated for: {}\n\n", name));
    }

    content.push_str("## Enabled Skills\n\n");
    for skill in enabled_skills {
        content.push_str(&format!("- {}\n", skill));
    }

    if !config.triggers.priority.is_empty() {
        content.push_str("\n## Trigger Priority\n\n");
        for (i, skill) in config.triggers.priority.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", i + 1, skill));
        }
    }

    if !config.triggers.overrides.is_empty() {
        content.push_str("\n## Trigger Overrides\n\n");
        content.push_str("| Trigger | Skill |\n");
        content.push_str("|---------|-------|\n");
        for (trigger, skill) in &config.triggers.overrides {
            content.push_str(&format!("| `{}` | {} |\n", trigger, skill));
        }
    }

    content
}

/// List available skill groups
pub fn execute_list_groups() -> Result<()> {
    let groups = get_skill_groups();

    println!("{}\n", "Available Skill Groups".bold().underline());

    let mut group_names: Vec<_> = groups.keys().collect();
    group_names.sort();

    for name in group_names {
        let skills = groups.get(name).unwrap();
        println!("{} ({} skills)", name.cyan().bold(), skills.len());
        for skill in skills {
            println!("  - {}", skill);
        }
        println!();
    }

    Ok(())
}

/// Sync Skills.lock with Skills.toml
///
/// Updates the lock file to reflect changes in the config file:
/// - Updates enabled/disabled status
/// - Removes packages that are no longer in config
/// - Syncs enabled status with Claude Code settings.json
/// - Optionally updates remote repos (git pull) with --update flag
pub fn execute_sync(project_root: &Path, update_remotes: bool) -> Result<()> {
    use std::process::Command;

    let config = load_config(project_root)?;
    let lock_path = skills_lock_path(project_root);

    // Load existing lock file
    let mut lock = match SkillsLock::load_from_project(project_root)? {
        Some(l) => l,
        None => {
            println!("{} No Skills.lock found. Run 'cowork config install' first.", "!".yellow());
            return Ok(());
        }
    };

    let mut remote_updated = 0;

    // Update remote repos if requested
    if update_remotes {
        println!("{}\n", "Updating remote repositories...".bold());

        let repos_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".cowork")
            .join("repos");

        for (name, dep) in &config.skills.install {
            // Only update GitHub repos
            if let Some(repo) = dep.repo() {
                let repo_name = repo.split('/').last().unwrap_or(name);
                let repo_dir = repos_dir.join(repo_name);

                if repo_dir.exists() {
                    print!("  {} {}... ", "→".blue(), name.cyan());
                    std::io::Write::flush(&mut std::io::stdout())?;

                    let output = Command::new("git")
                        .args(["pull", "--ff-only"])
                        .current_dir(&repo_dir)
                        .output()?;

                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.contains("Already up to date") {
                            println!("{}", "up to date".dimmed());
                        } else {
                            println!("{}", "updated".green());
                            remote_updated += 1;

                            // Update version in lock file
                            if let Some(pkg) = lock.package.iter_mut().find(|p| p.name == *name) {
                                // Get new version
                                if let Some(version) = get_plugin_version_from_dir(&repo_dir) {
                                    pkg.version = version;
                                }
                                // Get new git sha
                                if let Some(sha) = get_git_sha_from_dir(&repo_dir) {
                                    pkg.git_sha = Some(sha);
                                }
                                pkg.touch();
                            }
                        }
                    } else {
                        println!("{}", "failed".red());
                    }
                }
            }
        }
        println!();
    }

    println!("{}\n", "Syncing Skills.lock with Skills.toml".bold());

    let mut updated = 0;
    let removed;
    let mut settings_updates: Vec<(String, bool, InstallScope)> = Vec::new();

    // Build set of expected package names from config
    let mut expected_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for name in config.skills.install.keys() {
        expected_names.insert(name.clone());
    }
    for name in config.skills.dev.keys() {
        expected_names.insert(name.clone());
    }

    // Update enabled status for existing packages
    for pkg in &mut lock.package {
        // Check if package should be enabled based on config
        let should_be_enabled = if let Some(dep) = config.skills.install.get(&pkg.name) {
            dep.is_enabled()
        } else if let Some(link) = config.skills.dev.get(&pkg.name) {
            link.is_enabled()
        } else {
            // Package not in config - will be removed below
            continue;
        };

        if pkg.enabled != should_be_enabled {
            let status = if should_be_enabled { "enabled" } else { "disabled" };
            println!("  {} {} -> {}", "~".yellow(), pkg.name.cyan(), status);
            pkg.enabled = should_be_enabled;
            pkg.touch();
            updated += 1;

            // Track for settings.json update (only for plugins)
            if pkg.install_type == InstallType::Plugin {
                settings_updates.push((pkg.name.clone(), should_be_enabled, pkg.scope.clone()));
            }
        }
    }

    // Remove packages that are no longer in config
    let before_len = lock.package.len();
    lock.package.retain(|pkg| {
        if expected_names.contains(&pkg.name) {
            true
        } else {
            println!("  {} {} (removed from config)", "-".red(), pkg.name.dimmed());
            // Also remove from settings.json
            if pkg.install_type == InstallType::Plugin {
                settings_updates.push((pkg.name.clone(), false, pkg.scope.clone()));
            }
            false
        }
    });
    removed = before_len - lock.package.len();

    // Save lock file if changed
    if updated > 0 || removed > 0 || remote_updated > 0 {
        lock.save(&lock_path)?;
        println!("\n{} Updated {}", "✓".green(), lock_path.display());
        if remote_updated > 0 {
            println!("  {} remote updated, {} status changed, {} removed", remote_updated, updated, removed);
        } else {
            println!("  {} updated, {} removed", updated, removed);
        }
    } else {
        println!("\n{} Skills.lock is already in sync", "✓".green());
    }

    // Update Claude Code settings.json files
    if !settings_updates.is_empty() {
        println!("\n{}", "Syncing Claude Code settings...".bold());
        for (name, enabled, scope) in settings_updates {
            let plugin_id = format!("{}@{}", name, name);

            // Update global settings
            if let Err(e) = update_plugin_in_settings(&plugin_id, enabled) {
                println!("  {} Failed to update global settings: {}", "⚠".yellow(), e);
            } else {
                let status = if enabled { "enabled" } else { "disabled" };
                println!("  {} {} in global settings.json", "✓".green(), status);
            }

            // Update project settings if scope is project
            if scope == InstallScope::Project {
                if let Err(e) = update_plugin_in_project_settings(project_root, &plugin_id, enabled) {
                    println!("  {} Failed to update project settings: {}", "⚠".yellow(), e);
                } else {
                    let status = if enabled { "enabled" } else { "disabled" };
                    println!("  {} {} in project settings.json", "✓".green(), status);
                }
            }
        }
    }

    Ok(())
}

/// Update plugin enabled status in global settings.json
fn update_plugin_in_settings(plugin_id: &str, enabled: bool) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let settings_path = home.join(".claude").join("settings.json");

    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Update enabledPlugins
    if settings.get("enabledPlugins").is_none() {
        settings["enabledPlugins"] = serde_json::json!({});
    }
    settings["enabledPlugins"][plugin_id] = serde_json::Value::Bool(enabled);

    // Save
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_path, content)?;

    Ok(())
}

/// Update plugin enabled status in project settings.json
fn update_plugin_in_project_settings(project_root: &Path, plugin_id: &str, enabled: bool) -> Result<()> {
    let claude_dir = project_root.join(".claude");
    let settings_path = claude_dir.join("settings.json");

    fs::create_dir_all(&claude_dir)?;

    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Update enabledPlugins
    if settings.get("enabledPlugins").is_none() {
        settings["enabledPlugins"] = serde_json::json!({});
    }
    settings["enabledPlugins"][plugin_id] = serde_json::Value::Bool(enabled);

    // Save
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_path, content)?;

    Ok(())
}

/// Load config, creating default if not exists
fn load_config(project_root: &Path) -> Result<SkillsToml> {
    let config_path = skills_toml_path(project_root);

    if config_path.exists() {
        SkillsToml::load(&config_path)
    } else {
        // Return default config (will be created on save)
        Ok(SkillsToml::default())
    }
}

/// Save config to file
fn save_config(project_root: &Path, config: &SkillsToml) -> Result<()> {
    let config_path = skills_toml_path(project_root);

    // Create .cowork directory if needed
    let cowork_dir = project_root.join(COWORK_DIR);
    if !cowork_dir.exists() {
        fs::create_dir_all(&cowork_dir)?;
    }

    // If file doesn't exist, create with example format
    if !config_path.exists() {
        let content = SkillsToml::example_toml();
        fs::write(&config_path, content)?;
        // Now load and modify
        let mut new_config = SkillsToml::load(&config_path)?;
        new_config.skills = config.skills.clone();
        new_config.triggers = config.triggers.clone();
        new_config.save(&config_path)?;
    } else {
        config.save(&config_path)?;
    }

    Ok(())
}

/// Get plugin version from directory
fn get_plugin_version_from_dir(dir: &Path) -> Option<String> {
    // Try .claude-plugin/plugin.json first
    let plugin_json = dir.join(".claude-plugin").join("plugin.json");
    if plugin_json.exists() {
        if let Ok(content) = fs::read_to_string(&plugin_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                    return Some(version.to_string());
                }
            }
        }
    }

    // Try package.json
    let package_json = dir.join("package.json");
    if package_json.exists() {
        if let Ok(content) = fs::read_to_string(&package_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                    return Some(version.to_string());
                }
            }
        }
    }

    None
}

/// Get git SHA from directory
fn get_git_sha_from_dir(dir: &Path) -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Plugin info extracted from installed plugins
#[derive(Debug, Clone)]
struct PluginInfo {
    name: String,
    router: Option<String>,
    skills: Vec<SkillTriggerInfo>,
    keywords: Vec<String>,
}

/// Skill trigger info
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SkillTriggerInfo {
    name: String,
    triggers: Vec<String>,
}

// ============================================================================
// Extended Plugin Info for AI Analysis
// ============================================================================

/// Extended plugin info for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtendedPluginInfo {
    name: String,
    router: Option<String>,
    skills: Vec<ExtendedSkillInfo>,
    keywords: Vec<String>,
    readme_summary: Option<String>,
}

/// Extended skill info with full description
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtendedSkillInfo {
    name: String,
    triggers: Vec<String>,
    full_description: Option<String>,
    category: Option<String>,
}

/// AI analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisResult {
    version: String,
    plugins_hash: String,
    domains: Vec<DomainClassification>,
    layer_mapping: ThreeLayerMapping,
    skill_relationships: Vec<SkillRelationship>,
    cross_domain_routes: Vec<CrossDomainRoute>,
}

/// Domain classification from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomainClassification {
    domain: String,
    description: String,
    plugins: Vec<String>,
    primary_triggers: Vec<String>,
}

/// Three-layer mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThreeLayerMapping {
    layer3_why: Vec<LayerEntry>,
    layer2_what: Vec<LayerEntry>,
    layer1_how: Vec<LayerEntry>,
}

/// Entry in a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LayerEntry {
    trigger: String,
    skills: Vec<String>,
    description: String,
}

/// Skill relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillRelationship {
    from: String,
    to: String,
    relation: String,
}

/// Cross-domain routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrossDomainRoute {
    pattern: String,
    domains: Vec<String>,
    suggestion: String,
}

// ============================================================================
// Cache Management
// ============================================================================

/// Get cache directory path
fn get_cache_dir() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let cache_dir = home.join(".cowork").join("analysis-cache");
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }
    Ok(cache_dir)
}

/// Compute hash of plugins info for cache key
fn compute_plugins_hash(plugins: &[ExtendedPluginInfo]) -> String {
    let json = serde_json::to_string(plugins).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

/// Load cached analysis if valid
fn load_cached_analysis(plugins_hash: &str) -> Option<AnalysisResult> {
    let cache_dir = get_cache_dir().ok()?;
    let cache_file = cache_dir.join(format!("{}.json", plugins_hash));

    if !cache_file.exists() {
        return None;
    }

    // Check if cache is < 7 days old
    let metadata = fs::metadata(&cache_file).ok()?;
    let modified = metadata.modified().ok()?;
    let age = std::time::SystemTime::now()
        .duration_since(modified)
        .ok()?;

    if age.as_secs() > 7 * 24 * 60 * 60 {
        return None;
    }

    let content = fs::read_to_string(&cache_file).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save analysis to cache
fn save_analysis_cache(plugins_hash: &str, analysis: &AnalysisResult) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(format!("{}.json", plugins_hash));
    let content = serde_json::to_string_pretty(analysis)?;
    fs::write(cache_file, content)?;
    Ok(())
}

// ============================================================================
// Extended Info Collection
// ============================================================================

/// Extract extended plugin info for AI analysis
fn extract_extended_plugin_info(plugin_path: &Path, name: &str) -> Result<ExtendedPluginInfo> {
    let mut info = ExtendedPluginInfo {
        name: name.to_string(),
        router: None,
        skills: Vec::new(),
        keywords: Vec::new(),
        readme_summary: None,
    };

    // Extract README summary
    info.readme_summary = extract_readme_summary(plugin_path);

    // Check for router skill
    let router_names = [
        format!("{}-router", name.replace("-skills", "")),
        format!("{}-router", name),
        "router".to_string(),
    ];

    // Scan skills directory
    let skills_dir = plugin_path.join("skills");
    if skills_dir.exists() {
        if let Ok(entries) = fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let skill_md = path.join("SKILL.md");

                    if skill_md.exists() {
                        // Check if it's a router
                        if router_names.iter().any(|r| r == &skill_name) {
                            info.router = Some(skill_name.clone());
                        }

                        // Extract extended skill info
                        let extended_skill = extract_extended_skill_info(&skill_md, &skill_name);
                        info.keywords.extend(extended_skill.triggers.clone());
                        info.skills.push(extended_skill);
                    }
                }
            }
        }
    }

    // Also check for skills in .claude-plugin/skills/
    let plugin_skills_dir = plugin_path.join(".claude-plugin").join("skills");
    if plugin_skills_dir.exists() {
        if let Ok(entries) = fs::read_dir(&plugin_skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let skill_md = path.join("SKILL.md");

                    if skill_md.exists() {
                        let extended_skill = extract_extended_skill_info(&skill_md, &skill_name);
                        info.keywords.extend(extended_skill.triggers.clone());
                        info.skills.push(extended_skill);
                    }
                }
            }
        }
    }

    // Deduplicate keywords
    info.keywords.sort();
    info.keywords.dedup();

    Ok(info)
}

/// Extract README summary (first 100 lines)
fn extract_readme_summary(plugin_path: &Path) -> Option<String> {
    let readme_names = ["README.md", "readme.md", "README", "Readme.md"];

    for name in &readme_names {
        let readme_path = plugin_path.join(name);
        if readme_path.exists() {
            if let Ok(content) = fs::read_to_string(&readme_path) {
                let lines: Vec<&str> = content.lines().take(100).collect();
                return Some(lines.join("\n"));
            }
        }
    }
    None
}

/// Extract extended skill info from SKILL.md
fn extract_extended_skill_info(skill_md: &Path, name: &str) -> ExtendedSkillInfo {
    let mut skill = ExtendedSkillInfo {
        name: name.to_string(),
        triggers: Vec::new(),
        full_description: None,
        category: infer_skill_category(name),
    };

    if let Ok(content) = fs::read_to_string(skill_md) {
        // Truncate to ~1000 chars for the description
        skill.full_description = Some(content.chars().take(1000).collect());

        // Extract triggers
        if let Some(triggers) = extract_triggers_from_content(&content) {
            skill.triggers = triggers;
        }
    }

    skill
}

/// Extract triggers from content
fn extract_triggers_from_content(content: &str) -> Option<Vec<String>> {
    let triggers_pattern = regex::Regex::new(r#"(?i)triggers?\s*(?:on)?[:\s]+(.+?)(?:"|$|\n)"#).ok()?;

    if let Some(captures) = triggers_pattern.captures(content) {
        if let Some(triggers_str) = captures.get(1) {
            let triggers: Vec<String> = triggers_str.as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.len() > 1)
                .collect();
            if !triggers.is_empty() {
                return Some(triggers);
            }
        }
    }
    None
}

/// Infer skill category from name
fn infer_skill_category(name: &str) -> Option<String> {
    if name.starts_with("m0") || name.starts_with("m1") {
        Some("layer1".to_string())
    } else if name.contains("domain-") {
        Some("layer3".to_string())
    } else if name.contains("-router") || name.contains("meta-") {
        Some("layer2".to_string())
    } else {
        None
    }
}

// ============================================================================
// AI Analysis Module
// ============================================================================

/// Analysis prompt template for claude
const ANALYSIS_PROMPT_TEMPLATE: &str = r#"You are analyzing installed Claude Code plugins/skills to generate an intelligent router.

## Three-Layer Framework

The routing system uses a three-layer architecture:

### Layer 3: Domain Constraints (WHY)
- High-level domain rules and constraints
- Examples: web API patterns, fintech regulations, embedded limitations
- Triggers: domain keywords like "trading system", "web API", "embedded", "IoT"

### Layer 2: Design Choices (WHAT)
- Architectural decisions and design patterns
- Examples: "should I use Arc or Rc", "async vs sync", "error handling strategy"
- Triggers: design questions, pattern selection, architecture

### Layer 1: Implementation (HOW)
- Specific error codes and implementation details
- Examples: E0382, E0597, "borrowed value", "lifetime"
- Triggers: error codes, compiler messages, specific syntax

## Installed Plugins

```json
{plugins_json}
```

## Analysis Tasks

Analyze the installed plugins and provide:

1. **Domain Classification**: Group skills by domain (rust, dora, web, fintech, etc.)
2. **Three-Layer Trigger Mapping**: Map triggers to appropriate layers
3. **Skill Relationships**: Identify which skills work together
4. **Cross-Domain Routing**: Suggest when multiple domains apply

## Required Output Format

Return a JSON object with this exact structure:
```json
{
  "domains": [
    {
      "domain": "rust",
      "description": "Rust language mechanics and ecosystem",
      "plugins": ["rust-skills"],
      "primary_triggers": ["Rust", "cargo", "E0", "lifetime", "borrow"]
    }
  ],
  "layer_mapping": {
    "layer3_why": [
      {"trigger": "web API", "skills": ["domain-web"], "description": "Web domain constraints"}
    ],
    "layer2_what": [
      {"trigger": "design pattern", "skills": ["rust-router"], "description": "Design decisions"}
    ],
    "layer1_how": [
      {"trigger": "E0382", "skills": ["m01-ownership"], "description": "Ownership error"}
    ]
  },
  "skill_relationships": [
    {"from": "m01-ownership", "to": "domain-web", "relation": "L1→L3 context"}
  ],
  "cross_domain_routes": [
    {"pattern": "web.*E0382", "domains": ["rust", "web"], "suggestion": "Load both m01-ownership and domain-web"}
  ]
}
```

Important: Return ONLY the JSON object, no markdown code blocks or explanation."#;

/// Execute AI analysis using claude -p
fn execute_ai_analysis(plugins: &[ExtendedPluginInfo]) -> Result<AnalysisResult> {
    use std::process::Command;

    let plugins_json = serde_json::to_string_pretty(plugins)?;
    let prompt = ANALYSIS_PROMPT_TEMPLATE.replace("{plugins_json}", &plugins_json);

    println!("  {} Running AI analysis...", "→".blue());

    // Call claude -p
    let output = Command::new("claude")
        .args(["-p", &prompt])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("claude command failed: {}", stderr));
    }

    let response = String::from_utf8_lossy(&output.stdout);

    // Extract JSON from response
    let analysis = extract_json_from_response(&response)?;

    Ok(analysis)
}

/// Extract JSON from claude response
fn extract_json_from_response(response: &str) -> Result<AnalysisResult> {
    // Try to parse directly first
    if let Ok(result) = serde_json::from_str::<AnalysisResult>(response.trim()) {
        return Ok(result);
    }

    // Try to find JSON in markdown code blocks
    let json_pattern = regex::Regex::new(r"```(?:json)?\s*\n?([\s\S]*?)\n?```")?;

    for cap in json_pattern.captures_iter(response) {
        if let Some(json_str) = cap.get(1) {
            if let Ok(result) = serde_json::from_str::<AnalysisResult>(json_str.as_str().trim()) {
                return Ok(result);
            }
        }
    }

    // Try to find raw JSON object
    let obj_pattern = regex::Regex::new(r"\{[\s\S]*\}")?;
    if let Some(mat) = obj_pattern.find(response) {
        if let Ok(result) = serde_json::from_str::<AnalysisResult>(mat.as_str()) {
            return Ok(result);
        }
    }

    Err(anyhow::anyhow!("Could not extract valid JSON from AI response"))
}

// ============================================================================
// Enhanced Router Generation
// ============================================================================

/// Generate intelligent router skill with three-layer analysis
fn generate_intelligent_router_skill(plugins: &[PluginInfo], analysis: &AnalysisResult) -> String {
    let mut content = String::new();

    // Collect all keywords for the description
    let all_keywords: Vec<String> = plugins.iter()
        .flat_map(|p| p.keywords.iter().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .take(30)
        .collect();

    // Generate frontmatter
    content.push_str("---\n");
    content.push_str("name: cowork-router\n");
    content.push_str(&format!(
        "description: \"CRITICAL: Intelligent router with three-layer intent analysis. Triggers on: {}\"\n",
        all_keywords.join(", ")
    ));
    content.push_str("---\n\n");

    // Header
    content.push_str("# CoWork Intelligent Router\n\n");
    content.push_str("> AI-generated router with three-layer intent analysis\n\n");

    // Intent Detection Protocol
    content.push_str("## Intent Detection Protocol\n\n");
    content.push_str("### Step 1: Identify Entry Layer\n\n");
    content.push_str("| Signal | Entry Layer | Direction |\n");
    content.push_str("|--------|-------------|----------|\n");
    content.push_str("| Error code (E0xxx) | Layer 1 (HOW) | Trace UP ↑ |\n");
    content.push_str("| \"How to design...\" | Layer 2 (WHAT) | Check L3, then DOWN ↓ |\n");
    content.push_str("| \"Building a [domain] system\" | Layer 3 (WHY) | Trace DOWN ↓ |\n");
    content.push_str("\n");

    // Layer 3: Domain Constraints
    content.push_str("### Layer 3: Domain Constraints (WHY)\n\n");
    content.push_str("| Domain | Keywords | Skills | Route To |\n");
    content.push_str("|--------|----------|--------|----------|\n");

    for domain in &analysis.domains {
        let keywords = domain.primary_triggers.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
        let plugins_str = domain.plugins.join(", ");
        content.push_str(&format!(
            "| **{}** | {} | {} | `{}-router` |\n",
            domain.domain, keywords, plugins_str, domain.domain
        ));
    }

    if !analysis.layer_mapping.layer3_why.is_empty() {
        content.push_str("\n**Trigger Patterns:**\n\n");
        for entry in &analysis.layer_mapping.layer3_why {
            content.push_str(&format!(
                "- `{}` → {} ({})\n",
                entry.trigger,
                entry.skills.join(", "),
                entry.description
            ));
        }
    }
    content.push_str("\n");

    // Layer 2: Design Choices
    content.push_str("### Layer 2: Design Choices (WHAT)\n\n");

    if !analysis.layer_mapping.layer2_what.is_empty() {
        content.push_str("| Pattern | Skills | Description |\n");
        content.push_str("|---------|--------|-------------|\n");
        for entry in &analysis.layer_mapping.layer2_what {
            content.push_str(&format!(
                "| `{}` | {} | {} |\n",
                entry.trigger,
                entry.skills.join(", "),
                entry.description
            ));
        }
    } else {
        content.push_str("*Route design questions through domain routers*\n");
    }
    content.push_str("\n");

    // Layer 1: Implementation
    content.push_str("### Layer 1: Implementation (HOW)\n\n");

    if !analysis.layer_mapping.layer1_how.is_empty() {
        content.push_str("| Error/Pattern | Skills | Description |\n");
        content.push_str("|---------------|--------|-------------|\n");
        for entry in &analysis.layer_mapping.layer1_how {
            content.push_str(&format!(
                "| `{}` | {} | {} |\n",
                entry.trigger,
                entry.skills.join(", "),
                entry.description
            ));
        }
    } else {
        content.push_str("*Route implementation details through domain routers*\n");
    }
    content.push_str("\n");

    // Cross-Domain Routing
    if !analysis.cross_domain_routes.is_empty() {
        content.push_str("## Cross-Domain Routing\n\n");
        content.push_str("When multiple domains are detected, load skills from all relevant domains:\n\n");
        content.push_str("| Pattern | Domains | Suggestion |\n");
        content.push_str("|---------|---------|------------|\n");
        for route in &analysis.cross_domain_routes {
            content.push_str(&format!(
                "| `{}` | {} | {} |\n",
                route.pattern,
                route.domains.join(" + "),
                route.suggestion
            ));
        }
        content.push_str("\n");
    }

    // Skill Relationships
    if !analysis.skill_relationships.is_empty() {
        content.push_str("## Skill Relationships\n\n");
        content.push_str("```\n");
        for rel in &analysis.skill_relationships {
            content.push_str(&format!("{} --[{}]--> {}\n", rel.from, rel.relation, rel.to));
        }
        content.push_str("```\n\n");
    }

    // Architecture diagram
    content.push_str("## Architecture\n\n");
    content.push_str("```\n");
    content.push_str("┌───────────────────────────────────────────────────────────────────────┐\n");
    content.push_str("│                         cowork-router                                  │\n");
    content.push_str("│                   (Three-Layer Intent Analysis)                        │\n");
    content.push_str("└───────────────────────────────────────────────────────────────────────┘\n");
    content.push_str("        │\n");
    content.push_str("        ├── Layer 3 (WHY): Domain constraints\n");
    content.push_str("        ├── Layer 2 (WHAT): Design choices\n");
    content.push_str("        └── Layer 1 (HOW): Implementation details\n");
    content.push_str("                │\n");

    for (i, plugin) in plugins.iter().enumerate() {
        let is_last = i == plugins.len() - 1;
        let branch = if is_last { "└" } else { "├" };
        let router_name = plugin.router.as_deref().unwrap_or(&plugin.name);
        content.push_str(&format!("        {}──→ {} ({} skills)\n", branch, router_name, plugin.skills.len()));
    }

    content.push_str("```\n\n");

    // Installed Plugins section
    content.push_str("## Installed Plugins\n\n");

    for plugin in plugins {
        content.push_str(&format!("### {} ({} skills)\n\n", plugin.name, plugin.skills.len()));

        if let Some(router) = &plugin.router {
            content.push_str(&format!("**Router:** `{}`\n\n", router));
        }

        if !plugin.skills.is_empty() {
            let skill_names: Vec<_> = plugin.skills.iter().map(|s| s.name.as_str()).collect();
            content.push_str(&format!("**Skills:** {}\n\n", skill_names.join(", ")));
        }
    }

    // Usage section
    content.push_str("## Usage\n\n");
    content.push_str("This router uses three-layer intent analysis to route queries to the appropriate skills.\n\n");
    content.push_str("### Regenerate Router\n\n");
    content.push_str("```bash\n");
    content.push_str("# Quick generation (no AI)\n");
    content.push_str("cowork config router\n\n");
    content.push_str("# AI-enhanced generation (uses cache)\n");
    content.push_str("cowork config router --analyze\n\n");
    content.push_str("# Force re-analysis (ignore cache)\n");
    content.push_str("cowork config router --analyze --no-cache\n");
    content.push_str("```\n");

    content
}

/// Generate dynamic cowork-router based on Skills.toml configuration
pub fn execute_generate_router(
    project_root: &Path,
    with_hooks: bool,
    with_analysis: bool,
    no_cache: bool,
) -> Result<()> {
    let lock = match SkillsLock::load_from_project(project_root)? {
        Some(l) => l,
        None => {
            println!("{} No Skills.lock found. Run 'cowork config install' first.", "!".yellow());
            return Ok(());
        }
    };

    if with_analysis {
        println!("{}\n", "Generating AI-enhanced cowork-router...".bold());
    } else {
        println!("{}\n", "Generating dynamic cowork-router...".bold());
    }

    // Collect plugin info from installed packages
    let mut plugins: Vec<PluginInfo> = Vec::new();
    let mut extended_plugins: Vec<ExtendedPluginInfo> = Vec::new();

    for pkg in &lock.package {
        if !pkg.enabled {
            continue;
        }

        let install_path = std::path::Path::new(&pkg.install_path);
        if !install_path.exists() {
            continue;
        }

        // Check for plugin structure
        let plugin_info = extract_plugin_info(install_path, &pkg.name)?;
        if !plugin_info.skills.is_empty() || !plugin_info.keywords.is_empty() {
            plugins.push(plugin_info);

            // Also collect extended info for AI analysis
            if with_analysis {
                if let Ok(ext_info) = extract_extended_plugin_info(install_path, &pkg.name) {
                    extended_plugins.push(ext_info);
                }
            }
        }
    }

    if plugins.is_empty() {
        println!("{} No plugins with triggers found", "!".yellow());
        return Ok(());
    }

    // Generate router content
    let router_content = if with_analysis {
        // Try AI-enhanced generation
        match generate_router_with_analysis(&plugins, &extended_plugins, no_cache) {
            Ok(content) => content,
            Err(e) => {
                println!("{} AI analysis failed: {}. Falling back to basic generation.", "⚠".yellow(), e);
                generate_router_skill(&plugins)
            }
        }
    } else {
        generate_router_skill(&plugins)
    };

    // Write to .claude/skills/cowork-router/
    let router_dir = project_root.join(".claude").join("skills").join("cowork-router");
    fs::create_dir_all(&router_dir)?;
    let skill_path = router_dir.join("SKILL.md");
    fs::write(&skill_path, &router_content)?;

    println!("{} Generated {}", "✓".green(), skill_path.display());
    println!("  {} plugins with {} total skills", plugins.len(),
        plugins.iter().map(|p| p.skills.len()).sum::<usize>());

    // Generate hooks.json if requested
    if with_hooks {
        let hooks_content = generate_router_hooks(&plugins);
        let hooks_path = router_dir.join("hooks.json");
        fs::write(&hooks_path, &hooks_content)?;
        println!("{} Generated {}", "✓".green(), hooks_path.display());
    }

    Ok(())
}

/// Generate router with AI analysis
fn generate_router_with_analysis(
    plugins: &[PluginInfo],
    extended_plugins: &[ExtendedPluginInfo],
    no_cache: bool,
) -> Result<String> {
    // Compute hash for cache key
    let plugins_hash = compute_plugins_hash(extended_plugins);

    // Try to load from cache (unless no_cache is set)
    let analysis = if !no_cache {
        if let Some(cached) = load_cached_analysis(&plugins_hash) {
            println!("  {} Using cached analysis", "✓".green());
            cached
        } else {
            // Run AI analysis
            let result = execute_ai_analysis(extended_plugins)?;

            // Save to cache
            if let Err(e) = save_analysis_cache(&plugins_hash, &result) {
                println!("  {} Failed to cache analysis: {}", "⚠".yellow(), e);
            } else {
                println!("  {} Cached analysis for future use", "✓".green());
            }

            result
        }
    } else {
        println!("  {} Ignoring cache, running fresh analysis...", "→".blue());
        let result = execute_ai_analysis(extended_plugins)?;

        // Save to cache
        if let Err(e) = save_analysis_cache(&plugins_hash, &result) {
            println!("  {} Failed to cache analysis: {}", "⚠".yellow(), e);
        }

        result
    };

    Ok(generate_intelligent_router_skill(plugins, &analysis))
}

/// Extract plugin info from a plugin directory
fn extract_plugin_info(plugin_path: &Path, name: &str) -> Result<PluginInfo> {
    let mut info = PluginInfo {
        name: name.to_string(),
        router: None,
        skills: Vec::new(),
        keywords: Vec::new(),
    };

    // Check for router skill
    let router_names = [
        format!("{}-router", name.replace("-skills", "")),
        format!("{}-router", name),
        "router".to_string(),
    ];

    // Scan skills directory
    let skills_dir = plugin_path.join("skills");
    if skills_dir.exists() {
        if let Ok(entries) = fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let skill_md = path.join("SKILL.md");

                    if skill_md.exists() {
                        // Check if it's a router
                        if router_names.iter().any(|r| r == &skill_name) {
                            info.router = Some(skill_name.clone());
                        }

                        // Extract triggers from SKILL.md
                        if let Some(triggers) = extract_triggers_from_skill(&skill_md) {
                            info.keywords.extend(triggers.clone());
                            info.skills.push(SkillTriggerInfo {
                                name: skill_name,
                                triggers,
                            });
                        }
                    }
                }
            }
        }
    }

    // Also check for skills in .claude-plugin/skills/
    let plugin_skills_dir = plugin_path.join(".claude-plugin").join("skills");
    if plugin_skills_dir.exists() {
        if let Ok(entries) = fs::read_dir(&plugin_skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let skill_md = path.join("SKILL.md");

                    if skill_md.exists() {
                        if let Some(triggers) = extract_triggers_from_skill(&skill_md) {
                            info.keywords.extend(triggers.clone());
                            info.skills.push(SkillTriggerInfo {
                                name: skill_name,
                                triggers,
                            });
                        }
                    }
                }
            }
        }
    }

    // Deduplicate keywords
    info.keywords.sort();
    info.keywords.dedup();

    Ok(info)
}

/// Extract triggers from SKILL.md description
fn extract_triggers_from_skill(skill_md: &Path) -> Option<Vec<String>> {
    let content = fs::read_to_string(skill_md).ok()?;

    // Look for "Triggers on:" in the frontmatter description or content
    let triggers_pattern = regex::Regex::new(r#"(?i)triggers?\s*(?:on)?[:\s]+(.+?)(?:"|$|\n)"#).ok()?;

    if let Some(captures) = triggers_pattern.captures(&content) {
        if let Some(triggers_str) = captures.get(1) {
            let triggers: Vec<String> = triggers_str.as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.len() > 1)
                .collect();
            if !triggers.is_empty() {
                return Some(triggers);
            }
        }
    }

    None
}

/// Generate the router SKILL.md content
fn generate_router_skill(plugins: &[PluginInfo]) -> String {
    let mut content = String::new();

    // Collect all keywords for the description
    let all_keywords: Vec<String> = plugins.iter()
        .flat_map(|p| p.keywords.iter().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .take(30)  // Limit to 30 keywords
        .collect();

    // Generate frontmatter
    content.push_str("---\n");
    content.push_str("name: cowork-router\n");
    content.push_str(&format!(
        "description: \"CRITICAL: Unified router for installed plugins/skills. Triggers on: {}\"\n",
        all_keywords.join(", ")
    ));
    content.push_str("---\n\n");

    // Header
    content.push_str("# CoWork Router\n\n");
    content.push_str("> Auto-generated router based on Skills.toml configuration\n\n");

    // Architecture diagram
    content.push_str("## Architecture\n\n");
    content.push_str("```\n");
    content.push_str("┌───────────────────────────────────────────────────────────────────────┐\n");
    content.push_str("│                              cowork-router                             │\n");
    content.push_str("│                        (Unified Entry Point)                           │\n");
    content.push_str("└───────────────────────────────────────────────────────────────────────┘\n");
    content.push_str("                                        │\n");

    for (i, plugin) in plugins.iter().enumerate() {
        let is_last = i == plugins.len() - 1;
        let branch = if is_last { "└" } else { "├" };
        let router_name = plugin.router.as_deref().unwrap_or(&plugin.name);
        content.push_str(&format!("        {}──→ {} ({} skills)\n", branch, router_name, plugin.skills.len()));
    }

    content.push_str("```\n\n");

    // Domain Detection table
    content.push_str("## Domain Detection\n\n");
    content.push_str("| Domain | Keywords | Route To |\n");
    content.push_str("|--------|----------|----------|\n");

    for plugin in plugins {
        let router_name = plugin.router.as_deref().unwrap_or(&plugin.name);
        let keywords = plugin.keywords.iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let more = if plugin.keywords.len() > 5 { ", ..." } else { "" };
        content.push_str(&format!("| **{}** | {}{} | `{}` |\n",
            plugin.name, keywords, more, router_name));
    }

    content.push_str("\n");

    // Installed Skills section
    content.push_str("## Installed Skills\n\n");

    for plugin in plugins {
        content.push_str(&format!("### {} ({} skills)\n\n", plugin.name, plugin.skills.len()));

        if let Some(router) = &plugin.router {
            content.push_str(&format!("**Router:** `{}`\n\n", router));
        }

        if !plugin.skills.is_empty() {
            let skill_names: Vec<_> = plugin.skills.iter().map(|s| s.name.as_str()).collect();
            content.push_str(&format!("**Skills:** {}\n\n", skill_names.join(", ")));
        }
    }

    // Usage section
    content.push_str("## Usage\n\n");
    content.push_str("This router is auto-triggered when matching keywords are detected in your queries.\n\n");
    content.push_str("### Regenerate Router\n\n");
    content.push_str("```bash\n");
    content.push_str("# Regenerate after modifying Skills.toml\n");
    content.push_str("cowork config router\n\n");
    content.push_str("# Generate with hooks for auto-triggering\n");
    content.push_str("cowork config router --hooks\n");
    content.push_str("```\n");

    content
}

/// Generate hooks.json for auto-triggering
fn generate_router_hooks(plugins: &[PluginInfo]) -> String {
    // Collect all keywords for regex pattern
    let all_keywords: Vec<String> = plugins.iter()
        .flat_map(|p| p.keywords.iter().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Build regex pattern
    let pattern = format!("(?i)({})", all_keywords.join("|"));

    let hooks = serde_json::json!({
        "hooks": {
            "UserPromptSubmit": [{
                "matcher": pattern,
                "hooks": [{
                    "type": "skill",
                    "skill": "cowork-router"
                }]
            }]
        }
    });

    serde_json::to_string_pretty(&hooks).unwrap_or_default()
}
