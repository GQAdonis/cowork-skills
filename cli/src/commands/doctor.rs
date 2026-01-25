use crate::config::find_cowork_root;
use anyhow::Result;
use colored::Colorize;

pub fn execute() -> Result<()> {
    let cowork_root = find_cowork_root()?;

    println!("{}\n", "CoWork Skills Doctor".bold().underline());

    let mut issues = 0;
    let mut warnings = 0;

    // Check skills directory
    let skills_dir = cowork_root.join("skills");
    if !skills_dir.exists() {
        println!("{} skills/ directory not found", "✗".red());
        issues += 1;
    } else {
        println!("{} skills/ directory exists", "✓".green());

        // Count skills
        let skill_count = std::fs::read_dir(&skills_dir)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        !name.starts_with('.') && !name.starts_with('_')
                    })
                    .count()
            })
            .unwrap_or(0);
        println!("  {} skills found", skill_count);
    }

    // Check cowork-router
    println!("\n{}", "Checking router...".bold());
    let router_path = skills_dir.join("cowork-router");
    if !router_path.exists() {
        println!("  {} cowork-router not found", "⚠".yellow());
        warnings += 1;
    } else {
        let skill_md = router_path.join("SKILL.md");
        if !skill_md.exists() {
            println!("  {} cowork-router/SKILL.md not found", "⚠".yellow());
            warnings += 1;
        } else {
            println!("  {} cowork-router", "✓".green());
        }
    }

    // Check global skills path
    println!("\n{}", "Checking global skills...".bold());
    let global_path = dirs::home_dir()
        .map(|h| h.join(".claude").join("skills"));

    match global_path {
        Some(path) if path.exists() => {
            println!("  {} ~/.claude/skills/ exists", "✓".green());

            // Count global skills
            let skill_count = std::fs::read_dir(&path)
                .map(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                        .count()
                })
                .unwrap_or(0);
            println!("  {} global skills found", skill_count);
        }
        Some(_) => {
            println!("  {} ~/.claude/skills/ not found", "⚠".yellow());
            println!("  {} Run 'cowork init' to install built-in skills", "→".blue());
            warnings += 1;
        }
        None => {
            println!("  {} Could not determine home directory", "✗".red());
            issues += 1;
        }
    }

    // Summary
    println!("\n{}", "Summary:".bold());
    if issues == 0 && warnings == 0 {
        println!("  {} All checks passed!", "✓".green());
    } else {
        if issues > 0 {
            println!("  {} {} issue(s) found", "✗".red(), issues);
        }
        if warnings > 0 {
            println!("  {} {} warning(s) found", "⚠".yellow(), warnings);
        }
    }

    Ok(())
}
