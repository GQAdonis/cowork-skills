use crate::config::find_cowork_root;
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use walkdir::WalkDir;

pub fn execute(skill_type: &str, verbose: bool) -> Result<()> {
    let cowork_root = find_cowork_root()?;

    match skill_type {
        "project" => list_project_skills(&cowork_root, verbose)?,
        "global" => list_global_skills(verbose)?,
        "all" | _ => {
            list_project_skills(&cowork_root, verbose)?;
            println!();
            list_global_skills(verbose)?;
        }
    }

    Ok(())
}

fn list_project_skills(cowork_root: &std::path::Path, verbose: bool) -> Result<()> {
    println!("{}", "Project Skills:".bold().underline());

    let skills_dir = cowork_root.join("skills");
    if !skills_dir.exists() {
        println!("  No skills directory found.");
        return Ok(());
    }

    let mut skills: Vec<(String, Option<String>)> = Vec::new(); // (name, symlink_target)

    for entry in WalkDir::new(&skills_dir)
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

        if path.is_symlink() {
            let target = fs::read_link(path)
                .map(|t| t.to_string_lossy().to_string())
                .ok();
            skills.push((name, target));
        } else if path.is_dir() {
            skills.push((name, None));
        }
    }

    // Sort by name
    skills.sort_by(|a, b| a.0.cmp(&b.0));

    if skills.is_empty() {
        println!("  No skills found in skills/");
    } else {
        println!("\n  {} ({})", "skills/".cyan(), skills.len());
        for (name, target) in &skills {
            if verbose {
                if let Some(t) = target {
                    println!("    {} -> {}", name, t.dimmed());
                } else {
                    println!("    {}", name);
                }
            } else {
                println!("    {}", name);
            }
        }
        println!("\n  Total: {} skills", skills.len().to_string().green());
    }

    Ok(())
}

fn list_global_skills(verbose: bool) -> Result<()> {
    println!("{}", "Global Skills:".bold().underline());

    let global_path = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".claude")
        .join("skills");

    if !global_path.exists() {
        println!("  No global skills directory found at ~/.claude/skills/");
        return Ok(());
    }

    let mut skills: Vec<String> = Vec::new();

    for entry in WalkDir::new(&global_path)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == global_path {
            continue;
        }

        if path.is_dir() {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Skip hidden directories
            if !name.starts_with('.') {
                skills.push(name);
            }
        }
    }

    skills.sort();

    if skills.is_empty() {
        println!("  No skills found in ~/.claude/skills/");
    } else {
        println!("\n  {} ({})", "~/.claude/skills/".cyan(), skills.len());
        for skill in &skills {
            if verbose {
                println!("    {} (global)", skill);
            } else {
                println!("    {}", skill);
            }
        }
        println!("\n  Total: {} global skills", skills.len().to_string().green());
    }

    Ok(())
}
