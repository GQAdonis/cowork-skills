//! `cowork init` command - Install built-in skills to global ~/.claude/skills/

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

/// Built-in skills embedded at compile time
struct BuiltinSkill {
    name: &'static str,
    description: &'static str,
    content: &'static str,
}

/// List of all built-in skills
fn get_builtin_skills() -> Vec<BuiltinSkill> {
    vec![
        BuiltinSkill {
            name: "memory-skills",
            description: "CoALA cognitive architecture memory system",
            content: include_str!("../../builtin-skills/memory-skills/SKILL.md"),
        },
        BuiltinSkill {
            name: "cowork-guide",
            description: "CoWork CLI usage guide and documentation",
            content: include_str!("../../builtin-skills/cowork-guide/SKILL.md"),
        },
        BuiltinSkill {
            name: "cowork-router",
            description: "Unified router for installed plugins/skills",
            content: include_str!("../../builtin-skills/cowork-router/SKILL.md"),
        },
        BuiltinSkill {
            name: "code-review",
            description: "Code review assistant with best practices",
            content: include_str!("../../builtin-skills/code-review/SKILL.md"),
        },
        BuiltinSkill {
            name: "github-generate",
            description: "Generate skills from GitHub repositories",
            content: include_str!("../../builtin-skills/github-generate/SKILL.md"),
        },
        BuiltinSkill {
            name: "github-search",
            description: "Search GitHub for skill repositories",
            content: include_str!("../../builtin-skills/github-search/SKILL.md"),
        },
    ]
}

/// Get global skills directory
fn get_global_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".claude").join("skills"))
}

/// Get project-local skills directory
fn get_local_skills_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Could not get current directory")?;
    Ok(cwd.join(".claude").join("skills"))
}

/// Execute the init command
pub fn execute(list: bool, filter_skills: &[String], force: bool, local: bool, remove_skills: &[String]) -> Result<()> {
    let builtin_skills = get_builtin_skills();

    if list {
        println!("{}\n", "Available Built-in Skills".bold().underline());
        for skill in &builtin_skills {
            println!("  {} {}", "●".blue(), skill.name.cyan());
            println!("    {}", skill.description);
        }
        println!("\n{} Use 'cowork init' to install all (global)", "→".blue());
        println!("{} Use 'cowork init --local' to install to project", "→".blue());
        println!("{} Use 'cowork init -s <name>' to install specific skill", "→".blue());
        println!("{} Use 'cowork init -r <name>' to remove a skill", "→".blue());
        return Ok(());
    }

    let skills_dir = if local {
        get_local_skills_dir()?
    } else {
        get_global_skills_dir()?
    };

    let target_desc = if local {
        ".claude/skills/"
    } else {
        "~/.claude/skills/"
    };

    // Handle remove operation
    if !remove_skills.is_empty() {
        println!("{}\n", "Removing Skills".bold().underline());
        println!("{} Removing from {}\n", "→".blue(), target_desc);

        let mut removed = 0;
        let mut not_found = 0;

        for skill_name in remove_skills {
            let skill_dir = skills_dir.join(skill_name);
            if skill_dir.exists() {
                fs::remove_dir_all(&skill_dir)?;
                println!("  {} {}", "✓".green(), skill_name);
                removed += 1;
            } else {
                println!("  {} {} (not found)", "○".yellow(), skill_name);
                not_found += 1;
            }
        }

        println!("\n{}", "Summary:".bold());
        if removed > 0 {
            println!("  {} Removed: {}", "✓".green(), removed);
        }
        if not_found > 0 {
            println!("  {} Not found: {}", "○".yellow(), not_found);
        }
        return Ok(());
    }

    // Install operation
    println!("{}\n", "Initializing CoWork".bold().underline());
    fs::create_dir_all(&skills_dir)?;
    println!("{} Installing built-in skills to {}\n", "→".blue(), target_desc);

    let mut installed = 0;
    let mut skipped = 0;

    for skill in &builtin_skills {
        // Filter by skill names if specified
        if !filter_skills.is_empty() && !filter_skills.contains(&skill.name.to_string()) {
            continue;
        }

        let skill_dir = skills_dir.join(skill.name);
        let skill_file = skill_dir.join("SKILL.md");

        if skill_dir.exists() && !force {
            println!("  {} {} (already exists, use --force to overwrite)", "○".yellow(), skill.name);
            skipped += 1;
            continue;
        }

        // Create skill directory and write content
        fs::create_dir_all(&skill_dir)?;
        fs::write(&skill_file, skill.content)?;

        println!("  {} {}", "✓".green(), skill.name);
        installed += 1;
    }

    println!("\n{}", "Summary:".bold());
    if installed > 0 {
        println!("  {} Installed: {}", "✓".green(), installed);
    }
    if skipped > 0 {
        println!("  {} Skipped: {}", "○".yellow(), skipped);
    }

    if installed > 0 {
        println!("\n{} Skills installed to {}", "✓".green(), target_desc);
        if local {
            println!("{} These skills are available in the current project", "ℹ".blue());
        } else {
            println!("{} These skills are now available globally in Claude Code", "ℹ".blue());
        }
    }

    Ok(())
}
