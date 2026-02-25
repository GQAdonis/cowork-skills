use crate::agents::{detect_installed_agents, get_agent_names, get_all_agents};
use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Install options
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub repo: Option<String>,
    pub uninstall: bool,
    pub list: bool,
    /// Force reinstall (remove existing and install fresh)
    pub reinstall: bool,
    /// Update to latest version (git pull + reinstall)
    pub update: bool,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub plugin: bool,
    pub include_dirs: Vec<String>,
    pub no_symlink: bool,
    pub yes: bool,
    pub use_add_skill: bool,
    /// Install to current project instead of global
    pub local: bool,
}

/// Information about an installation for lock file
#[derive(Debug, Clone, Default)]
pub struct InstallInfo {
    /// Version string (from plugin.json, package.json, or timestamp)
    pub version: Option<String>,
    /// Git commit SHA
    pub git_sha: Option<String>,
    /// Installation path
    pub install_path: String,
    /// List of installed skills
    pub skills: Vec<String>,
}

/// Get installation info for lock file
pub fn get_install_info(options: &InstallOptions, project_root: &Path) -> Result<InstallInfo> {
    let repos_dir = get_repos_dir()?;

    let source = options.repo.as_ref().ok_or_else(|| {
        anyhow::anyhow!("No repo specified")
    })?;

    // Determine if this is a local path or GitHub repo
    let path = Path::new(source);
    let (repo_dir, name) = if path.exists() && path.is_dir() {
        // Local path
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "local-skills".to_string());
        (path.to_path_buf(), name)
    } else if source.contains('/') {
        // GitHub repo
        let (_, name, _) = parse_repo(source)?;
        (repos_dir.join(&name), name)
    } else {
        return Err(anyhow::anyhow!("Invalid source: {}", source));
    };

    // Get version from plugin or package
    let version = get_plugin_version(&repo_dir);

    // Get git sha
    let git_sha = get_git_sha(&repo_dir);

    // Determine install path
    let install_path = if options.local {
        if options.plugin {
            project_root.join(".claude").join(&name)
        } else {
            project_root.join(".claude").join("skills")
        }
    } else {
        let home = dirs::home_dir().context("Could not find home directory")?;
        if options.plugin {
            home.join(".claude").join(&name)
        } else {
            home.join(".claude").join("skills")
        }
    };

    // Get list of installed skills
    let skills = if options.plugin {
        // For plugins, list skills from the skills/ directory
        let skills_dir = repo_dir.join("skills");
        list_skill_names(&skills_dir)
    } else if options.skills.is_empty() {
        // All skills from skills/ directory
        let skills_dir = repo_dir.join("skills");
        list_skill_names(&skills_dir)
    } else {
        options.skills.clone()
    };

    Ok(InstallInfo {
        version,
        git_sha,
        install_path: install_path.to_string_lossy().to_string(),
        skills,
    })
}

/// List skill names from a skills directory
fn list_skill_names(skills_dir: &Path) -> Vec<String> {
    if !skills_dir.exists() {
        return Vec::new();
    }

    WalkDir::new(skills_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            let path = e.path();
            path != skills_dir
                && path.is_dir()
                && !e.file_name().to_string_lossy().starts_with('.')
                && !e.file_name().to_string_lossy().starts_with('_')
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}

/// Get the cowork repos directory (~/.cowork/repos/)
fn get_repos_dir() -> Result<PathBuf> {
    let repos_dir = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".cowork")
        .join("repos");
    Ok(repos_dir)
}

/// Parse GitHub repo string (user/repo or full URL)
fn parse_repo(repo: &str) -> Result<(String, String, Option<String>)> {
    let repo = repo.trim_end_matches('/').trim_end_matches(".git");

    // Check for subpath in GitHub URL (e.g., /tree/main/skills/frontend-design)
    let mut subpath = None;
    let repo_part = if repo.contains("/tree/") {
        let parts: Vec<&str> = repo.split("/tree/").collect();
        if parts.len() == 2 {
            let path_parts: Vec<&str> = parts[1].splitn(2, '/').collect();
            if path_parts.len() == 2 {
                subpath = Some(path_parts[1].to_string());
            }
        }
        parts[0]
    } else {
        repo
    };

    if repo_part.contains("github.com") || repo_part.contains("gitlab.com") {
        let parts: Vec<&str> = repo_part.split('/').collect();
        if parts.len() >= 2 {
            let user = parts[parts.len() - 2].to_string();
            let name = parts[parts.len() - 1].to_string();
            return Ok((user, name, subpath));
        }
    } else if repo_part.contains('/') && !repo_part.contains(':') {
        let parts: Vec<&str> = repo_part.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string(), subpath));
        }
    }

    bail!("Invalid repo format. Use 'user/repo' or full GitHub URL")
}

pub fn execute(options: InstallOptions) -> Result<()> {
    // Use add-skill if requested
    if options.use_add_skill {
        return execute_with_add_skill(&options);
    }

    let repos_dir = get_repos_dir()?;

    if options.list {
        return list_installed(&repos_dir);
    }

    // Determine target agents
    let target_agents = if options.agents.is_empty() {
        let detected = detect_installed_agents();
        if detected.is_empty() {
            vec!["claude-code"] // Default to claude-code
        } else {
            detected
        }
    } else {
        // Validate agent names
        let valid_agents = get_agent_names();
        for agent in &options.agents {
            if !valid_agents.contains(&agent.as_str()) {
                bail!(
                    "Unknown agent: {}. Valid agents: {}",
                    agent,
                    valid_agents.join(", ")
                );
            }
        }
        options.agents.iter().map(|s| s.as_str()).collect()
    };

    match (&options.repo, options.uninstall, options.reinstall, options.update) {
        // Uninstall
        (Some(repo), true, _, _) => {
            if options.local {
                uninstall_local(repo)
            } else {
                uninstall_repo(repo, &repos_dir, &target_agents)
            }
        }

        // Reinstall: uninstall first, then install
        (Some(repo), false, true, _) => {
            println!("{} Reinstalling {}...\n", "→".blue(), repo.cyan());

            // Remove existing installation first
            let name = if repo.contains('/') {
                parse_repo(repo)?.1
            } else {
                repo.to_string()
            };

            // Remove from local project if --local
            if options.local {
                let project_root = std::env::current_dir()?;
                let plugin_dir = project_root.join(".claude").join(&name);
                if plugin_dir.exists() {
                    fs::remove_dir_all(&plugin_dir)?;
                    println!("  {} Removed existing installation", "✓".green());
                }
            }

            // Now install fresh
            let path = Path::new(repo);
            if path.exists() && path.is_dir() {
                install_from_local_path(path, &target_agents, &options)
            } else {
                install_from_github(repo, &repos_dir, &target_agents, &options)
            }
        }

        // Update: git pull then reinstall
        (Some(repo), false, _, true) => {
            println!("{} Updating {}...\n", "→".blue(), repo.cyan());

            let name = if repo.contains('/') {
                parse_repo(repo)?.1
            } else {
                repo.to_string()
            };

            // Update cached repo
            let repo_dir = repos_dir.join(&name);
            if repo_dir.exists() {
                println!("  {} Pulling latest changes...", "→".blue());
                let output = Command::new("git")
                    .args(["pull", "--ff-only"])
                    .current_dir(&repo_dir)
                    .output()?;

                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains("Already up to date") {
                        println!("  {} Already up to date", "✓".green());
                    } else {
                        println!("  {} Updated to latest", "✓".green());
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("  {} Failed to pull: {}", "⚠".yellow(), stderr.trim());
                }
            }

            // Reinstall with updated content
            let mut reinstall_options = options.clone();
            reinstall_options.reinstall = true;
            reinstall_options.update = false;

            let path = Path::new(repo);
            if path.exists() && path.is_dir() {
                install_from_local_path(path, &target_agents, &reinstall_options)
            } else {
                install_from_github(repo, &repos_dir, &target_agents, &reinstall_options)
            }
        }

        // Normal install
        (Some(repo), false, false, false) => {
            let path = Path::new(repo);
            if path.exists() && path.is_dir() {
                install_from_local_path(path, &target_agents, &options)
            } else {
                install_from_github(repo, &repos_dir, &target_agents, &options)
            }
        }

        (None, true, _, _) => {
            println!("Usage: cowork install --uninstall <repo-name>");
            println!("       cowork install --list  # to see installed repos");
            Ok(())
        }
        (None, _, true, _) | (None, _, _, true) => {
            println!("Usage: cowork install <repo> --reinstall");
            println!("       cowork install <repo> --update");
            Ok(())
        }
        (None, false, false, false) => install_current_project(&target_agents, &options),
    }
}

/// Install skills from a local path
fn install_from_local_path(
    local_path: &Path,
    target_agents: &[&str],
    options: &InstallOptions,
) -> Result<()> {
    let name = local_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "local-skills".to_string());

    println!(
        "{} {} from local path...\n",
        "Installing".bold(),
        name.cyan()
    );

    // Check if this is a plugin (has .claude-plugin directory)
    let is_plugin = local_path.join(".claude-plugin").exists();

    // Plugin mode
    if options.plugin {
        if !is_plugin {
            println!("  {} Directory is not a plugin (no .claude-plugin directory)", "⚠".yellow());
            println!("  {} Falling back to skills installation", "→".blue());
        } else {
            // Install as plugin
            if options.local {
                return install_plugin_local_mode(local_path, &name, options);
            } else {
                return install_plugin_mode(local_path, &name, target_agents, options);
            }
        }
    }

    // Check if this is a single skill (has SKILL.md directly)
    let is_single_skill = local_path.join("SKILL.md").exists();

    if is_single_skill {
        // Install single skill directly
        return install_single_skill(local_path, &name, target_agents, options);
    }

    // Skills mode - check for skills directory
    let skills_dir = local_path.join("skills");
    if !skills_dir.exists() {
        bail!(
            "No skills/ directory or SKILL.md found in {}. Not a valid skills project or skill.",
            local_path.display()
        );
    }

    // Local skills installation mode
    if options.local {
        return install_local_mode(&skills_dir, &name, options);
    }

    // Global installation to agents
    let agents = get_all_agents();
    let mut total_installed = 0;
    let mut total_skipped = 0;

    for agent_name in target_agents {
        if let Some(agent) = agents.get(agent_name) {
            println!("\n  {} Installing to {}...", "→".blue(), agent.display_name);

            let target_dir = &agent.global_skills_dir;
            fs::create_dir_all(target_dir)?;

            let (installed, skipped) = install_skills_from_dir(
                &skills_dir,
                target_dir,
                &options.skills,
                options.no_symlink,
            )?;

            total_installed += installed;
            total_skipped += skipped;
        }
    }

    println!("\n{}", "Summary:".bold());
    println!("  {} Installed: {}", "✓".green(), total_installed);
    if total_skipped > 0 {
        println!("  {} Skipped: {}", "○".blue(), total_skipped);
    }
    println!(
        "\n{} {} skills installed to {} agent(s)",
        "✓".green(),
        name.cyan(),
        target_agents.len()
    );

    Ok(())
}

/// Install a single skill directory (contains SKILL.md)
fn install_single_skill(
    skill_path: &Path,
    name: &str,
    target_agents: &[&str],
    options: &InstallOptions,
) -> Result<()> {
    println!(
        "  {} Detected single skill: {}",
        "✓".green(),
        name.cyan()
    );

    let agents = get_all_agents();

    if options.local {
        // Install to project local
        let project_root = std::env::current_dir()?;
        let target_dir = project_root.join(".claude").join("skills").join(name);
        fs::create_dir_all(&target_dir)?;

        // Copy skill contents
        copy_dir_contents(skill_path, &target_dir)?;

        println!(
            "\n{} Installed {} to .claude/skills/{}",
            "✓".green(),
            name.cyan(),
            name
        );
    } else {
        // Install to all target agents
        for agent_name in target_agents {
            if let Some(agent) = agents.get(agent_name) {
                println!("\n  {} Installing to {}...", "→".blue(), agent.display_name);

                let target_dir = agent.global_skills_dir.join(name);
                fs::create_dir_all(&target_dir)?;

                // Copy skill contents
                copy_dir_contents(skill_path, &target_dir)?;

                println!("    {} Installed: {}", "✓".green(), name);
            }
        }

        println!(
            "\n{} {} installed to {} agent(s)",
            "✓".green(),
            name.cyan(),
            target_agents.len()
        );
    }

    Ok(())
}

/// Copy directory contents recursively, following symlinks
fn copy_dir_contents(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src).min_depth(1).follow_links(true) {
        let entry = entry?;
        let src_path = entry.path();
        let relative = src_path.strip_prefix(src)?;
        let dst_path = dst.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dst_path)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn execute_with_add_skill(options: &InstallOptions) -> Result<()> {
    println!(
        "{} Using add-skill backend...\n",
        "→".blue()
    );

    // Check if add-skill is available
    let check = Command::new("npx")
        .args(["add-skill", "--version"])
        .output();

    if check.is_err() {
        bail!("add-skill not found. Install with: npm install -g add-skill");
    }

    let mut args = vec!["add-skill".to_string()];

    // Add repo if provided
    if let Some(repo) = &options.repo {
        args.push(repo.clone());
    }

    // Add options
    if options.uninstall {
        // add-skill doesn't have uninstall, use internal
        println!("{} add-skill doesn't support uninstall, using internal handler", "⚠".yellow());
        return execute(InstallOptions {
            use_add_skill: false,
            ..options.clone()
        });
    }

    if options.list {
        args.push("--list".to_string());
    }

    for skill in &options.skills {
        args.push("--skill".to_string());
        args.push(skill.clone());
    }

    for agent in &options.agents {
        args.push("--agent".to_string());
        args.push(agent.clone());
    }

    if options.plugin {
        args.push("--plugin".to_string());
    }

    for dir in &options.include_dirs {
        args.push("--include-dir".to_string());
        args.push(dir.clone());
    }

    if options.no_symlink {
        args.push("--no-symlink".to_string());
    }

    if options.yes {
        args.push("--yes".to_string());
    }

    // Add global flag
    args.push("--global".to_string());

    // Execute npx add-skill
    let status = Command::new("npx")
        .args(&args)
        .status()
        .context("Failed to run npx add-skill")?;

    if !status.success() {
        bail!("add-skill failed");
    }

    Ok(())
}

fn install_from_github(
    repo: &str,
    repos_dir: &Path,
    target_agents: &[&str],
    options: &InstallOptions,
) -> Result<()> {
    let (user, name, subpath) = parse_repo(repo)?;
    let repo_url = format!("https://github.com/{}/{}.git", user, name);
    let repo_dir = repos_dir.join(&name);

    println!(
        "{} {} from GitHub...\n",
        "Installing".bold(),
        format!("{}/{}", user, name).cyan()
    );

    // Create repos directory
    fs::create_dir_all(repos_dir).context("Failed to create ~/.cowork/repos/")?;

    // Clone or update repo
    if repo_dir.exists() {
        println!("  {} Updating existing repo...", "→".blue());
        let status = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(&repo_dir)
            .status()
            .context("Failed to run git pull")?;

        if !status.success() {
            println!(
                "  {} Git pull failed, continuing with existing version",
                "⚠".yellow()
            );
        }
    } else {
        println!("  {} Cloning {}...", "→".blue(), repo_url);
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                &repo_url,
                repo_dir.to_str().unwrap(),
            ])
            .status()
            .context("Failed to run git clone")?;

        if !status.success() {
            bail!("Failed to clone repository: {}", repo_url);
        }
    }

    // Determine skills directory
    let skills_dir = if let Some(ref sp) = subpath {
        repo_dir.join(sp)
    } else {
        repo_dir.join("skills")
    };

    // Check if repo is a plugin (has .claude-plugin directory)
    let is_plugin = repo_dir.join(".claude-plugin").exists();

    // Plugin mode: copy entire repo structure
    if options.plugin {
        if !is_plugin {
            println!(
                "  {} Repository is not a plugin (no .claude-plugin directory)",
                "⚠".yellow()
            );
            println!("  {} Falling back to skills installation", "→".blue());
        } else {
            return install_plugin_mode(&repo_dir, &name, target_agents, options);
        }
    }

    // Auto-detect: if it's a plugin and no explicit mode, suggest plugin install
    if is_plugin && !options.plugin && !options.local {
        println!(
            "  {} Detected plugin structure. Use --plugin for full plugin install.",
            "ℹ".blue()
        );
    }

    if !skills_dir.exists() {
        if is_plugin {
            // It's a plugin without skills/ dir, install as plugin
            println!(
                "  {} No skills/ directory, installing as plugin...",
                "→".blue()
            );
            return install_plugin_mode(&repo_dir, &name, target_agents, options);
        }
        bail!(
            "No skills/ directory found in {}. Not a valid skills repository.",
            name
        );
    }

    // Local (project) installation mode
    if options.local {
        return install_local_mode(&skills_dir, &name, options);
    }

    // Install skills to each target agent
    let agents = get_all_agents();
    let mut total_installed = 0;
    let mut total_skipped = 0;

    for agent_name in target_agents {
        if let Some(agent) = agents.get(agent_name) {
            println!("\n  {} Installing to {}...", "→".blue(), agent.display_name);

            let target_dir = &agent.global_skills_dir;
            fs::create_dir_all(target_dir)?;

            let (installed, skipped) = install_skills_from_dir(
                &skills_dir,
                target_dir,
                &options.skills,
                options.no_symlink,
            )?;

            // Install include dirs
            for include_dir in &options.include_dirs {
                let src_dir = repo_dir.join(include_dir);
                if src_dir.exists() {
                    let parent = target_dir.parent().unwrap_or(target_dir);
                    let dest_dir = parent.join(include_dir);
                    copy_directory(&src_dir, &dest_dir, options.no_symlink)?;
                    println!("    {} {} (included)", "+".green(), include_dir);
                }
            }

            total_installed += installed;
            total_skipped += skipped;
        }
    }

    println!("\n{}", "Summary:".bold());
    println!("  {} Installed: {}", "✓".green(), total_installed);
    if total_skipped > 0 {
        println!("  {} Skipped: {}", "○".blue(), total_skipped);
    }
    println!(
        "\n{} {} skills installed to {} agent(s)",
        "✓".green(),
        name.cyan(),
        target_agents.len()
    );

    Ok(())
}

/// Install skills to current project (local mode)
fn install_local_mode(skills_dir: &Path, name: &str, options: &InstallOptions) -> Result<()> {
    let project_root = std::env::current_dir()?;

    // Always use .claude/skills/ for Claude Code project skills
    // This is the standard location for project-scoped skills
    let target_dir = project_root.join(".claude").join("skills");

    fs::create_dir_all(&target_dir)?;

    println!("\n  {} Installing to project: {}", "→".blue(), target_dir.display());

    let (installed, skipped) = install_skills_from_dir(
        skills_dir,
        &target_dir,
        &options.skills,
        options.no_symlink,
    )?;

    println!("\n{}", "Summary:".bold());
    println!("  {} Installed: {}", "✓".green(), installed);
    if skipped > 0 {
        println!("  {} Skipped: {}", "○".blue(), skipped);
    }
    println!(
        "\n{} {} skills installed to project",
        "✓".green(),
        name.cyan()
    );
    println!("  Location: {}", target_dir.display());

    Ok(())
}

fn install_plugin_mode(
    repo_dir: &Path,
    name: &str,
    target_agents: &[&str],
    options: &InstallOptions,
) -> Result<()> {
    // Local plugin mode: install to project's .claude/<plugin-name>/
    if options.local {
        return install_plugin_local_mode(repo_dir, name, options);
    }

    println!("\n  {} Installing as plugin (full structure)...", "→".blue());

    let agents = get_all_agents();
    let mut installed_path: Option<PathBuf> = None;

    for agent_name in target_agents {
        if let Some(agent) = agents.get(agent_name) {
            // For plugin mode, install to parent of skills dir
            let base_dir = agent
                .global_skills_dir
                .parent()
                .unwrap_or(&agent.global_skills_dir);
            let plugin_dir = base_dir.join(name);

            // Remove existing directory if it exists (to avoid permission issues)
            if plugin_dir.exists() {
                fs::remove_dir_all(&plugin_dir).context("Failed to remove existing plugin directory")?;
            }

            fs::create_dir_all(&plugin_dir)?;

            // Copy/symlink entire repo structure
            copy_directory(repo_dir, &plugin_dir, options.no_symlink)?;

            // Track first installation path for registration
            if installed_path.is_none() {
                installed_path = Some(plugin_dir.clone());
            }

            println!(
                "    {} {} -> {} ({})",
                "+".green(),
                name,
                plugin_dir.display(),
                agent.display_name
            );
        }
    }

    // Register plugin in Claude Code's system
    if let Some(install_path) = installed_path {
        if let Err(e) = register_global_plugin(name, &install_path, repo_dir) {
            println!(
                "    {} Failed to register plugin: {}",
                "⚠".yellow(),
                e
            );
        } else {
            println!("    {} Registered in Claude Code", "✓".green());
        }

        if let Err(e) = enable_plugin_in_settings(&format!("{}@{}", name, name)) {
            println!(
                "    {} Failed to enable plugin: {}",
                "⚠".yellow(),
                e
            );
        } else {
            println!("    {} Enabled in settings.json", "✓".green());
        }
    }

    println!(
        "\n{} {} installed as plugin to {} agent(s)",
        "✓".green(),
        name.cyan(),
        target_agents.len()
    );

    Ok(())
}

/// Install plugin to project's .claude/<plugin-name>/ directory
fn install_plugin_local_mode(
    repo_dir: &Path,
    name: &str,
    options: &InstallOptions,
) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let plugin_dir = project_root.join(".claude").join(name);

    println!("\n  {} Installing as local plugin...", "→".blue());

    // Remove existing directory if it exists (to avoid permission issues)
    if plugin_dir.exists() {
        fs::remove_dir_all(&plugin_dir).context("Failed to remove existing plugin directory")?;
    }

    fs::create_dir_all(&plugin_dir)?;

    // Copy/symlink entire repo structure
    copy_directory(repo_dir, &plugin_dir, options.no_symlink)?;

    println!(
        "    {} {} -> {}",
        "+".green(),
        name,
        plugin_dir.display()
    );

    // Register as marketplace (required for plugin discovery)
    if let Err(e) = register_as_marketplace(name, repo_dir) {
        println!(
            "    {} Failed to register marketplace: {}",
            "⚠".yellow(),
            e
        );
    } else {
        println!("    {} Registered marketplace", "✓".green());
    }

    // Register in Claude Code's installed_plugins.json
    if let Err(e) = register_local_plugin(name, &plugin_dir, &project_root) {
        println!(
            "    {} Failed to register plugin: {}",
            "⚠".yellow(),
            e
        );
    } else {
        println!("    {} Registered in Claude Code", "✓".green());
    }

    // Enable in global settings.json
    if let Err(e) = enable_plugin_in_settings(&format!("{}@{}", name, name)) {
        println!(
            "    {} Failed to enable in global settings: {}",
            "⚠".yellow(),
            e
        );
    } else {
        println!("    {} Enabled in global settings.json", "✓".green());
    }

    // Enable in project settings.json (required for local plugins)
    if let Err(e) = enable_plugin_in_project_settings(&project_root, &format!("{}@{}", name, name)) {
        println!(
            "    {} Failed to enable in project settings: {}",
            "⚠".yellow(),
            e
        );
    } else {
        println!("    {} Enabled in project .claude/settings.json", "✓".green());
    }

    println!(
        "\n{} {} installed as local plugin",
        "✓".green(),
        name.cyan()
    );
    println!("  Location: {}", plugin_dir.display());
    println!("\n{} Restart Claude Code to load the plugin", "ℹ".blue());

    Ok(())
}

/// Register a local plugin in Claude Code's installed_plugins.json
fn register_local_plugin(name: &str, install_path: &Path, project_path: &Path) -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let plugins_dir = home.join(".claude").join("plugins");
    let installed_path = plugins_dir.join("installed_plugins.json");

    fs::create_dir_all(&plugins_dir)?;

    // Load existing or create new
    let mut installed: serde_json::Value = if installed_path.exists() {
        let content = fs::read_to_string(&installed_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({
            "version": 2,
            "plugins": {}
        })
    };

    // Get version and git sha from plugin
    let version = get_plugin_version(install_path).unwrap_or_else(|| "1.0.0".to_string());
    let git_sha = get_git_sha(install_path);

    // Create plugin ID
    let plugin_id = format!("{}@{}", name, name);

    // Create installation entry
    let now = chrono::Utc::now().to_rfc3339();
    let mut installation = serde_json::json!({
        "scope": "local",
        "projectPath": project_path.to_string_lossy(),
        "installPath": install_path.to_string_lossy(),
        "version": version,
        "installedAt": now,
        "lastUpdated": now
    });

    // Add git commit sha if available
    if let Some(sha) = git_sha {
        installation["gitCommitSha"] = serde_json::Value::String(sha);
    }

    // Add to plugins
    if let Some(plugins) = installed.get_mut("plugins") {
        if let Some(obj) = plugins.as_object_mut() {
            obj.insert(plugin_id, serde_json::json!([installation]));
        }
    }

    // Save
    let content = serde_json::to_string_pretty(&installed)?;
    fs::write(&installed_path, content)?;

    Ok(())
}

/// Get plugin version from plugin.json, package.json, or manifest.json
fn get_plugin_version(plugin_dir: &Path) -> Option<String> {
    // Try .claude-plugin/plugin.json first (standard plugin format)
    let plugin_json = plugin_dir.join(".claude-plugin").join("plugin.json");
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
    let package_json = plugin_dir.join("package.json");
    if package_json.exists() {
        if let Ok(content) = fs::read_to_string(&package_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                    return Some(version.to_string());
                }
            }
        }
    }

    // Try .claude-plugin/manifest.json (legacy)
    let manifest = plugin_dir.join(".claude-plugin").join("manifest.json");
    if manifest.exists() {
        if let Ok(content) = fs::read_to_string(&manifest) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                    return Some(version.to_string());
                }
            }
        }
    }

    None
}

/// Enable plugin in Claude Code's global settings.json
fn enable_plugin_in_settings(plugin_id: &str) -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let settings_path = home.join(".claude").join("settings.json");

    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Add to enabledPlugins
    if settings.get("enabledPlugins").is_none() {
        settings["enabledPlugins"] = serde_json::json!({});
    }
    settings["enabledPlugins"][plugin_id] = serde_json::Value::Bool(true);

    // Save
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_path, content)?;

    Ok(())
}

/// Enable plugin in project's .claude/settings.json (required for local plugins)
fn enable_plugin_in_project_settings(project_path: &Path, plugin_id: &str) -> Result<()> {
    let claude_dir = project_path.join(".claude");
    let settings_path = claude_dir.join("settings.json");

    fs::create_dir_all(&claude_dir)?;

    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Add to enabledPlugins
    if settings.get("enabledPlugins").is_none() {
        settings["enabledPlugins"] = serde_json::json!({});
    }
    settings["enabledPlugins"][plugin_id] = serde_json::Value::Bool(true);

    // Save
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_path, content)?;

    Ok(())
}

/// Register a global plugin in Claude Code's installed_plugins.json
fn register_global_plugin(name: &str, install_path: &Path, repo_dir: &Path) -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let plugins_dir = home.join(".claude").join("plugins");
    let installed_json_path = plugins_dir.join("installed_plugins.json");

    fs::create_dir_all(&plugins_dir)?;

    // Load existing or create new
    let mut installed: serde_json::Value = if installed_json_path.exists() {
        let content = fs::read_to_string(&installed_json_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({
            "version": 2,
            "plugins": {}
        })
    };

    // Get version and git sha from plugin
    let version = get_plugin_version(repo_dir).unwrap_or_else(|| "1.0.0".to_string());
    let git_sha = get_git_sha(repo_dir);

    // Create plugin ID
    let plugin_id = format!("{}@{}", name, name);

    // Create installation entry
    let now = chrono::Utc::now().to_rfc3339();
    let mut installation = serde_json::json!({
        "scope": "user",
        "installPath": install_path.to_string_lossy(),
        "version": version,
        "installedAt": now,
        "lastUpdated": now
    });

    if let Some(sha) = git_sha {
        installation["gitCommitSha"] = serde_json::Value::String(sha);
    }

    // Add to plugins
    if let Some(plugins) = installed.get_mut("plugins") {
        if let Some(obj) = plugins.as_object_mut() {
            obj.insert(plugin_id, serde_json::json!([installation]));
        }
    }

    // Save
    let content = serde_json::to_string_pretty(&installed)?;
    fs::write(&installed_json_path, content)?;

    // Also register as marketplace for discoverability
    register_as_marketplace(name, repo_dir)?;

    Ok(())
}

/// Register plugin repo as a marketplace
fn register_as_marketplace(name: &str, repo_dir: &Path) -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let plugins_dir = home.join(".claude").join("plugins");
    let marketplaces_dir = plugins_dir.join("marketplaces");
    let known_path = plugins_dir.join("known_marketplaces.json");

    // Copy to marketplaces directory
    let marketplace_dir = marketplaces_dir.join(name);
    if !marketplace_dir.exists() {
        fs::create_dir_all(&marketplace_dir)?;
        copy_directory(repo_dir, &marketplace_dir, false)?;
    }

    // Update known_marketplaces.json
    let mut known: serde_json::Value = if known_path.exists() {
        let content = fs::read_to_string(&known_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Get git remote URL
    let remote_url = get_git_remote(repo_dir);
    let repo_str = remote_url
        .as_ref()
        .and_then(|url| extract_github_repo(url))
        .unwrap_or_else(|| format!("local/{}", name));

    let now = chrono::Utc::now().to_rfc3339();
    known[name] = serde_json::json!({
        "source": {
            "source": "github",
            "repo": repo_str
        },
        "installLocation": marketplace_dir.to_string_lossy(),
        "lastUpdated": now
    });

    let content = serde_json::to_string_pretty(&known)?;
    fs::write(&known_path, content)?;

    Ok(())
}

/// Get git commit SHA from repo
fn get_git_sha(repo_dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Get git remote URL from repo
fn get_git_remote(repo_dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_dir)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Extract GitHub repo from URL (e.g., "https://github.com/user/repo.git" -> "user/repo")
fn extract_github_repo(url: &str) -> Option<String> {
    let url = url.trim_end_matches(".git");
    if url.contains("github.com") {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            return Some(format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]));
        }
    }
    None
}

fn install_current_project(target_agents: &[&str], options: &InstallOptions) -> Result<()> {
    let cowork_root = crate::config::find_cowork_root()?;
    let skills_dir = cowork_root.join("skills");

    if !skills_dir.exists() {
        bail!("No skills/ directory found in current project");
    }

    println!(
        "{}\n",
        "Installing current project skills...".bold()
    );

    let agents = get_all_agents();
    let mut total_installed = 0;
    let mut total_skipped = 0;

    for agent_name in target_agents {
        if let Some(agent) = agents.get(agent_name) {
            println!("  {} Installing to {}...", "→".blue(), agent.display_name);

            let target_dir = &agent.global_skills_dir;
            fs::create_dir_all(target_dir)?;

            let (installed, skipped) = install_skills_from_dir(
                &skills_dir,
                target_dir,
                &options.skills,
                options.no_symlink,
            )?;

            total_installed += installed;
            total_skipped += skipped;
        }
    }

    println!("\n{}", "Summary:".bold());
    println!("  {} Installed: {}", "✓".green(), total_installed);
    println!("  {} Skipped: {}", "○".blue(), total_skipped);
    println!(
        "\n{} Skills installed to {} agent(s)",
        "✓".green(),
        target_agents.len()
    );

    Ok(())
}

fn install_skills_from_dir(
    skills_dir: &Path,
    target_dir: &Path,
    filter_skills: &[String],
    no_symlink: bool,
) -> Result<(usize, usize)> {
    let mut installed = 0;
    let mut skipped = 0;

    for entry in WalkDir::new(skills_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == skills_dir {
            continue;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Skip hidden and special directories
        if name.starts_with('.') || name.starts_with('_') {
            continue;
        }

        if !path.is_dir() && !path.is_symlink() {
            continue;
        }

        // Filter by skill names if specified
        if !filter_skills.is_empty() && !filter_skills.contains(&name) {
            continue;
        }

        let target = target_dir.join(&name);

        // Get the absolute path to the skill
        let skill_path = if path.is_symlink() {
            let link_target = fs::read_link(path)?;
            if link_target.is_absolute() {
                link_target
            } else {
                skills_dir.join(&link_target).canonicalize()?
            }
        } else {
            path.canonicalize()?
        };

        if target.exists() {
            println!("    {} {} (skipped - exists)", "○".blue(), name);
            skipped += 1;
            continue;
        }

        if no_symlink {
            // Copy instead of symlink
            copy_directory(&skill_path, &target, true)?;
            println!("    {} {} (copied)", "+".green(), name);
        } else {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&skill_path, &target)?;
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(&skill_path, &target)?;

            println!("    {} {}", "+".green(), name);
        }
        installed += 1;
    }

    Ok((installed, skipped))
}

fn copy_directory(src: &Path, dst: &Path, _recursive: bool) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    // follow_links(true) ensures symlinks to directories are recursed into
    // and their contents are copied as real files/dirs
    for entry in WalkDir::new(src).follow_links(true).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let relative = path.strip_prefix(src)?;
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            // Remove existing file/symlink first to avoid permission issues
            if target.exists() || target.is_symlink() {
                fs::remove_file(&target).ok(); // Ignore errors, copy will fail if needed
            }
            fs::copy(path, &target)?;
        }
    }

    Ok(())
}

/// Uninstall locally installed plugin or skills
fn uninstall_local(name: &str) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let claude_dir = project_root.join(".claude");

    println!("{} {} from project...\n", "Uninstalling".bold(), name.cyan());

    let mut removed = false;

    // Check for plugin installation (.claude/<name>/)
    let plugin_dir = claude_dir.join(name);
    if plugin_dir.exists() {
        fs::remove_dir_all(&plugin_dir)?;
        println!("  {} Removed plugin: .claude/{}", "-".red(), name);
        removed = true;

        // Also remove from project settings.json
        let settings_path = claude_dir.join("settings.json");
        if settings_path.exists() {
            if let Ok(content) = fs::read_to_string(&settings_path) {
                if let Ok(mut settings) = serde_json::from_str::<serde_json::Value>(&content) {
                    let plugin_id = format!("{}@{}", name, name);
                    if let Some(enabled) = settings.get_mut("enabledPlugins") {
                        if let Some(obj) = enabled.as_object_mut() {
                            obj.remove(&plugin_id);
                            let new_content = serde_json::to_string_pretty(&settings)?;
                            fs::write(&settings_path, new_content)?;
                            println!("  {} Removed from project settings.json", "-".red());
                        }
                    }
                }
            }
        }
    }

    // Check for skills installation (.claude/skills/<name>/)
    let skills_dir = claude_dir.join("skills");
    if skills_dir.exists() {
        let skill_path = skills_dir.join(name);
        if skill_path.exists() || skill_path.is_symlink() {
            if skill_path.is_dir() && !skill_path.is_symlink() {
                fs::remove_dir_all(&skill_path)?;
            } else {
                fs::remove_file(&skill_path)?;
            }
            println!("  {} Removed skill: .claude/skills/{}", "-".red(), name);
            removed = true;
        }
    }

    if removed {
        println!("\n{} {} uninstalled from project", "✓".green(), name.cyan());
    } else {
        println!("  {} {} not found in project", "⚠".yellow(), name);
        println!("\n  Checked locations:");
        println!("    - .claude/{}/", name);
        println!("    - .claude/skills/{}/", name);
    }

    Ok(())
}

fn uninstall_repo(repo: &str, repos_dir: &Path, target_agents: &[&str]) -> Result<()> {
    let name = if repo.contains('/') {
        parse_repo(repo)?.1
    } else {
        repo.to_string()
    };

    let repo_dir = repos_dir.join(&name);

    println!("{} {}...\n", "Uninstalling".bold(), name.cyan());

    let agents = get_all_agents();
    let mut total_removed = 0;

    // Remove from each target agent
    for agent_name in target_agents {
        if let Some(agent) = agents.get(agent_name) {
            let skills_dir = repo_dir.join("skills");
            let global_skills = &agent.global_skills_dir;

            if skills_dir.exists() && global_skills.exists() {
                for entry in WalkDir::new(&skills_dir)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(Result::ok)
                {
                    let path = entry.path();
                    if path == skills_dir {
                        continue;
                    }

                    let skill_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if skill_name.starts_with('.') || skill_name.starts_with('_') {
                        continue;
                    }

                    let target = global_skills.join(&skill_name);

                    if target.exists() || target.is_symlink() {
                        if target.is_dir() && !target.is_symlink() {
                            fs::remove_dir_all(&target)?;
                        } else {
                            fs::remove_file(&target)?;
                        }
                        println!(
                            "  {} {} (removed from {})",
                            "-".red(),
                            skill_name,
                            agent.display_name
                        );
                        total_removed += 1;
                    }
                }
            }

            // Also check for plugin mode installation
            let base_dir = agent
                .global_skills_dir
                .parent()
                .unwrap_or(&agent.global_skills_dir);
            let plugin_dir = base_dir.join(&name);

            if plugin_dir.exists() {
                fs::remove_dir_all(&plugin_dir)?;
                println!(
                    "  {} {} (removed plugin from {})",
                    "-".red(),
                    name,
                    agent.display_name
                );
                total_removed += 1;
            }
        }
    }

    // Remove the repo directory
    if repo_dir.exists() {
        fs::remove_dir_all(&repo_dir).context("Failed to remove repo directory")?;
        println!("  {} {} (removed repo)", "-".red(), name);
    }

    println!("\n{}", "Summary:".bold());
    println!("  {} Removed: {}", "-".red(), total_removed);

    Ok(())
}

fn list_installed(repos_dir: &Path) -> Result<()> {
    println!("{}\n", "Installed skill repositories:".bold().underline());

    if !repos_dir.exists() {
        println!("  (no repositories installed)");
        println!("\n  Install with: cowork install <user/repo>");
        return Ok(());
    }

    let mut count = 0;
    for entry in fs::read_dir(repos_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let skills_dir = path.join("skills");

        let skill_count = if skills_dir.exists() {
            WalkDir::new(&skills_dir)
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| {
                    let p = e.path();
                    p != skills_dir
                        && p.is_dir()
                        && !e.file_name().to_string_lossy().starts_with('.')
                })
                .count()
        } else {
            0
        };

        let remote = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&path)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        println!("  {} {} ({} skills)", "●".cyan(), name.bold(), skill_count);
        if !remote.is_empty() {
            println!("    {}", remote.dimmed());
        }

        count += 1;
    }

    if count == 0 {
        println!("  (no repositories installed)");
    }

    println!("\n  Total: {} repositories", count);

    // Show detected agents
    let detected = detect_installed_agents();
    if !detected.is_empty() {
        println!("\n{}", "Detected agents:".bold());
        for agent in detected {
            println!("  {} {}", "●".green(), agent);
        }
    }

    Ok(())
}
