//! `cowork status` command - Show current configuration status.

use crate::config::{find_cowork_root, get_claude_dir};
use crate::skills_toml::{skills_toml_path, SkillsLock, SkillsToml};
use anyhow::Result;
use colored::Colorize;
use std::fs;

pub fn execute() -> Result<()> {
    let cowork_root = find_cowork_root()?;

    println!("{}\n", "CoWork Skills Status".bold().underline());

    // Root directory
    println!("{}: {}", "Project Root".cyan(), cowork_root.display());

    // Skills.toml status
    let config_path = skills_toml_path(&cowork_root);
    if config_path.exists() {
        println!("{}: {}", "Config".cyan(), "Skills.toml".green());

        if let Ok(config) = SkillsToml::load(&config_path) {
            if let Some(name) = &config.project.name {
                println!("{}: {}", "Project Name".cyan(), name);
            }

            // Install dependencies
            if !config.skills.install.is_empty() {
                println!("\n{}", "Install Dependencies:".bold());
                for (name, dep) in &config.skills.install {
                    let repo = dep.repo().unwrap_or("(local)");
                    println!("  {} {} -> {}", "●".blue(), name, repo);
                }
            }

            // Dev links
            if !config.skills.dev.is_empty() {
                println!("\n{}", "Dev Links:".bold());
                for (name, link) in &config.skills.dev {
                    let path = link.path();
                    println!("  {} {} -> {}", "●".yellow(), name, path);
                }
            }

            // Enabled/disabled
            if !config.skills.global.enabled.is_empty() {
                println!("\n{}", "Enabled Skills:".bold());
                for skill in &config.skills.global.enabled {
                    println!("  {} {}", "✓".green(), skill);
                }
            }
        }
    } else {
        println!(
            "{}: {} (run 'cowork config init' to create)",
            "Config".cyan(),
            "not found".yellow()
        );
    }

    // Skills.lock status
    if let Ok(Some(lock)) = SkillsLock::load_from_project(&cowork_root) {
        println!("\n{}", "Locked Packages:".bold());
        for pkg in &lock.package {
            let status = if pkg.enabled {
                "✓".green()
            } else {
                "○".yellow()
            };
            println!(
                "  {} {} v{} ({:?})",
                status,
                pkg.name,
                pkg.version,
                pkg.scope
            );
        }
    }

    // Global skills directory
    println!("\n{}", "Global Skills:".bold());
    if let Ok(claude_dir) = get_claude_dir() {
        let skills_dir = claude_dir.join("skills");
        if skills_dir.exists() {
            let mut count = 0;
            for entry in fs::read_dir(&skills_dir)? {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                if entry.path().join("SKILL.md").exists() {
                    count += 1;
                }
            }
            println!(
                "  {}: {} skills",
                "~/.claude/skills/".cyan(),
                count.to_string().green()
            );
        } else {
            println!(
                "  {} ~/.claude/skills/ not found (run 'cowork init')",
                "!".yellow()
            );
        }
    }

    // Project skills directory
    let project_skills = cowork_root.join(".claude").join("skills");
    if project_skills.exists() {
        let mut count = 0;
        for entry in fs::read_dir(&project_skills)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if entry.path().join("SKILL.md").exists() {
                count += 1;
            }
        }
        println!(
            "  {}: {} skills",
            ".claude/skills/".cyan(),
            count.to_string().green()
        );
    }

    Ok(())
}
