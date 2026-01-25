//! Security audit command for CoWork Skills.
//!
//! Scans installed skills for:
//! - Dangerous patterns (rm -rf, eval, etc.)
//! - Prompt injection attempts
//! - Credential leaks
//! - Version drift from remote

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config::get_claude_dir;
use crate::security::{scan_skills_directory_with_config, AuditReport, RiskLevel, SecurityConfig};
use crate::skills_toml::{SkillsLock, SkillsToml};

/// Options for the audit command
pub struct AuditOptions {
    /// Scan global skills (~/.claude/skills/)
    pub global: bool,
    /// Scan project skills (.claude/skills/)
    pub project: bool,
    /// Scan plugins
    pub plugins: bool,
    /// Output format (text, json, markdown)
    pub format: String,
    /// Output file path
    pub output: Option<PathBuf>,
    /// Show verbose output
    pub verbose: bool,
    /// Fix issues automatically (where possible)
    #[allow(dead_code)]
    pub fix: bool,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            global: true,
            project: true,
            plugins: true,
            format: "text".to_string(),
            output: None,
            verbose: false,
            fix: false,
        }
    }
}

/// Execute the audit command
pub fn execute(options: AuditOptions) -> Result<()> {
    println!("🔍 CoWork Security Audit");
    println!("========================\n");

    let mut report = AuditReport::new();

    // Load security config if available
    let project_root = std::env::current_dir()?;
    let security_config = load_security_config(&project_root);

    // Show skip paths if configured
    if options.verbose {
        if !security_config.skip_paths.is_empty() {
            println!("Skip paths configured:");
            for path in &security_config.skip_paths {
                println!("  - {}", path);
            }
            println!();
        }
        if !security_config.trusted_marketplaces.is_empty() {
            println!("Trusted marketplaces:");
            for marketplace in &security_config.trusted_marketplaces {
                println!("  - {}", marketplace);
            }
            println!();
        }
    }

    // Scan global skills
    if options.global {
        let global_skills_dir = get_claude_dir()?.join("skills");
        if global_skills_dir.exists() {
            println!("Scanning global skills: {}", global_skills_dir.display());
            let results =
                scan_skills_directory_with_config(&global_skills_dir, Some(&security_config))?;
            for result in results {
                report.add_result(result);
            }
        }
    }

    // Scan project skills
    if options.project {
        let project_skills_dir = project_root.join(".claude").join("skills");
        if project_skills_dir.exists() {
            println!("Scanning project skills: {}", project_skills_dir.display());
            let results =
                scan_skills_directory_with_config(&project_skills_dir, Some(&security_config))?;
            for result in results {
                report.add_result(result);
            }
        }
    }

    // Scan plugins
    if options.plugins {
        let plugins_dir = get_claude_dir()?;
        if plugins_dir.exists() {
            // Scan each plugin directory (excluding 'skills' subdirectory)
            for entry in std::fs::read_dir(&plugins_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && path.file_name().map(|n| n != "skills").unwrap_or(false) {
                    // Check if this plugin directory should be skipped
                    if security_config.should_skip_path(&path) {
                        if options.verbose {
                            println!("Skipping trusted plugin: {}", path.display());
                        }
                        continue;
                    }

                    if options.verbose {
                        println!("Scanning plugin: {}", path.display());
                    }
                    let results =
                        scan_skills_directory_with_config(&path, Some(&security_config))?;
                    for result in results {
                        report.add_result(result);
                    }
                }
            }
        }
    }

    // Check version drift if lockfile exists
    check_version_drift(&project_root, &mut report)?;

    // Print report
    println!();
    print_report(&report, &options, &security_config)?;

    // Save to file if requested
    if let Some(output_path) = &options.output {
        save_report(&report, output_path, &options.format)?;
        println!("\n📄 Report saved to: {}", output_path.display());
    }

    // Exit with error if critical/high issues found
    if report.has_critical_issues() {
        std::process::exit(2);
    } else if report.has_high_issues() {
        std::process::exit(1);
    }

    Ok(())
}

fn load_security_config(project_root: &Path) -> SecurityConfig {
    if let Ok(Some(config)) = SkillsToml::load_from_project(project_root) {
        config.security
    } else {
        SecurityConfig::default()
    }
}

fn check_version_drift(project_root: &Path, report: &mut AuditReport) -> Result<()> {
    if let Ok(Some(lock)) = SkillsLock::load_from_project(project_root) {
        for package in &lock.package {
            // For now, just record what we have
            // In future, could query GitHub for remote SHA
            let drift = crate::security::VersionDrift {
                skill_name: package.name.clone(),
                installed_version: package.version.clone(),
                installed_sha: package.git_sha.clone(),
                remote_version: None, // Would need GitHub API call
                remote_sha: None,
                has_drift: false, // Can't determine without remote check
            };
            report.version_drifts.push(drift);
        }
    }
    Ok(())
}

fn print_report(report: &AuditReport, options: &AuditOptions, security_config: &SecurityConfig) -> Result<()> {
    println!("📊 Audit Summary");
    println!("────────────────");
    println!("Skills scanned: {}", report.scanned_skills);
    println!("Total issues:   {}", report.total_issues);
    println!();
    
    if report.total_issues > 0 {
        println!("Issues by severity:");
        if report.critical_count > 0 {
            println!("  🔴 CRITICAL: {}", report.critical_count);
        }
        if report.high_count > 0 {
            println!("  🟠 HIGH:     {}", report.high_count);
        }
        if report.medium_count > 0 {
            println!("  🟡 MEDIUM:   {}", report.medium_count);
        }
        if report.low_count > 0 {
            println!("  🟢 LOW:      {}", report.low_count);
        }
        println!();
    }
    
    // Print details for each skill with issues
    for result in &report.results {
        if result.issues.is_empty() {
            if options.verbose {
                println!("✅ {} - No issues found", result.skill_name);
                println!("   Hash: {}", result.content_hash);
            }
            continue;
        }
        
        let risk_icon = match result.overall_risk {
            RiskLevel::Critical => "🔴",
            RiskLevel::High => "🟠",
            RiskLevel::Medium => "🟡",
            RiskLevel::Low => "🟢",
            RiskLevel::Safe => "✅",
        };
        
        println!("{} {} - {} issue(s)", risk_icon, result.skill_name, result.issues.len());
        println!("   Path: {}", result.skill_path);
        println!("   Hash: {}", result.content_hash);
        
        // Check if from trusted author
        if let Some(author) = extract_author_from_path(&result.skill_path) {
            if security_config.is_trusted_author(&author) {
                println!("   Author: {} (trusted)", author);
            } else {
                println!("   Author: {} (untrusted)", author);
            }
        }
        
        println!("   Issues:");
        for issue in &result.issues {
            let level_str = match issue.risk_level {
                RiskLevel::Critical => "\x1b[31mCRITICAL\x1b[0m",
                RiskLevel::High => "\x1b[33mHIGH\x1b[0m",
                RiskLevel::Medium => "\x1b[33mMEDIUM\x1b[0m",
                RiskLevel::Low => "\x1b[32mLOW\x1b[0m",
                RiskLevel::Safe => "SAFE",
            };
            
            print!("   - [{}] {}", level_str, issue.description);
            if let Some(line) = issue.line_number {
                print!(" (line {})", line);
            }
            println!();
            
            if options.verbose {
                println!("     Pattern: {}", issue.pattern);
                if let Some(ctx) = &issue.context {
                    println!("     Context: {}...", ctx);
                }
            }
        }
        println!();
    }
    
    // Print version drift warnings
    if !report.version_drifts.is_empty() && options.verbose {
        println!("📦 Installed Packages");
        println!("────────────────────");
        for drift in &report.version_drifts {
            println!("  {} v{}", drift.skill_name, drift.installed_version);
            if let Some(sha) = &drift.installed_sha {
                println!("    SHA: {}", &sha[..8.min(sha.len())]);
            }
        }
        println!();
    }
    
    // Final recommendation
    if report.has_critical_issues() {
        println!("⛔ CRITICAL issues detected! Immediate action required.");
        println!("   Recommendation: Remove affected skills and investigate source.");
    } else if report.has_high_issues() {
        println!("⚠️  HIGH risk issues detected. Review carefully before use.");
    } else if report.total_issues > 0 {
        println!("ℹ️  Some issues detected. Review if concerning.");
    } else {
        println!("✅ No security issues detected.");
    }
    
    Ok(())
}

fn extract_author_from_path(path: &str) -> Option<String> {
    // Try to extract author from path patterns like:
    // ~/.cowork/repos/user/repo/...
    // ~/.claude/plugin_user@repo/...
    
    if path.contains("repos/") {
        let parts: Vec<&str> = path.split("repos/").collect();
        if parts.len() > 1 {
            let after_repos = parts[1];
            if let Some(author) = after_repos.split('/').next() {
                return Some(author.to_string());
            }
        }
    }
    
    None
}

fn save_report(report: &AuditReport, path: &Path, format: &str) -> Result<()> {
    let content = match format {
        "json" => serde_json::to_string_pretty(report)?,
        "markdown" => format_report_markdown(report),
        _ => format_report_text(report),
    };
    
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write report to {}", path.display()))?;
    
    Ok(())
}

fn format_report_markdown(report: &AuditReport) -> String {
    let mut out = String::new();
    
    out.push_str("# CoWork Security Audit Report\n\n");
    out.push_str(&format!("**Date:** {}\n\n", report.timestamp));
    
    out.push_str("## Summary\n\n");
    out.push_str(&format!("| Metric | Count |\n"));
    out.push_str("|--------|-------|\n");
    out.push_str(&format!("| Skills Scanned | {} |\n", report.scanned_skills));
    out.push_str(&format!("| Total Issues | {} |\n", report.total_issues));
    out.push_str(&format!("| Critical | {} |\n", report.critical_count));
    out.push_str(&format!("| High | {} |\n", report.high_count));
    out.push_str(&format!("| Medium | {} |\n", report.medium_count));
    out.push_str(&format!("| Low | {} |\n\n", report.low_count));
    
    if !report.results.is_empty() {
        out.push_str("## Detailed Findings\n\n");
        
        for result in &report.results {
            if result.issues.is_empty() {
                continue;
            }
            
            out.push_str(&format!("### {} ({:?})\n\n", result.skill_name, result.overall_risk));
            out.push_str(&format!("- **Path:** `{}`\n", result.skill_path));
            out.push_str(&format!("- **Hash:** `{}`\n\n", result.content_hash));
            
            out.push_str("| Risk | Description | Line |\n");
            out.push_str("|------|-------------|------|\n");
            
            for issue in &result.issues {
                out.push_str(&format!(
                    "| {:?} | {} | {} |\n",
                    issue.risk_level,
                    issue.description,
                    issue.line_number.map(|n| n.to_string()).unwrap_or_default()
                ));
            }
            out.push_str("\n");
        }
    }
    
    out
}

fn format_report_text(report: &AuditReport) -> String {
    let mut out = String::new();
    
    out.push_str("CoWork Security Audit Report\n");
    out.push_str("============================\n\n");
    out.push_str(&format!("Date: {}\n\n", report.timestamp));
    out.push_str(&format!("Skills Scanned: {}\n", report.scanned_skills));
    out.push_str(&format!("Total Issues: {}\n", report.total_issues));
    out.push_str(&format!("  Critical: {}\n", report.critical_count));
    out.push_str(&format!("  High: {}\n", report.high_count));
    out.push_str(&format!("  Medium: {}\n", report.medium_count));
    out.push_str(&format!("  Low: {}\n\n", report.low_count));
    
    for result in &report.results {
        if result.issues.is_empty() {
            continue;
        }
        
        out.push_str(&format!("{} ({:?})\n", result.skill_name, result.overall_risk));
        out.push_str(&format!("  Path: {}\n", result.skill_path));
        out.push_str(&format!("  Hash: {}\n", result.content_hash));
        
        for issue in &result.issues {
            out.push_str(&format!(
                "  - [{:?}] {} (line {:?})\n",
                issue.risk_level,
                issue.description,
                issue.line_number
            ));
        }
        out.push_str("\n");
    }
    
    out
}
