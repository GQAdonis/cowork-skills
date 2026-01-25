//! Verify command for CoWork Skills checksum validation.
//!
//! Verifies installed skills against their recorded checksums in Skills.lock.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::get_claude_dir;
use crate::security::hash_file;
use crate::skills_toml::{InstallScope, SkillsLock};

/// Options for the verify command
pub struct VerifyOptions {
    /// Specific skill to verify (if None, verify all)
    pub skill: Option<String>,
    /// Update checksums in lockfile
    pub update: bool,
    /// Show verbose output
    pub verbose: bool,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            skill: None,
            update: false,
            verbose: false,
        }
    }
}

/// Verification result for a single skill
#[derive(Debug)]
pub struct VerifyResult {
    pub skill_name: String,
    pub path: PathBuf,
    pub expected_hash: Option<String>,
    pub actual_hash: String,
    pub is_valid: bool,
    pub is_missing: bool,
}

/// Execute the verify command
pub fn execute(options: VerifyOptions) -> Result<()> {
    println!("🔐 CoWork Checksum Verification");
    println!("================================\n");

    let project_root = std::env::current_dir()?;
    
    // Load lockfile
    let lock = match SkillsLock::load_from_project(&project_root)? {
        Some(lock) => lock,
        None => {
            println!("⚠️  No Skills.lock found in project.");
            println!("   Run 'cowork config install' first to create lockfile.");
            return Ok(());
        }
    };
    
    let mut results: Vec<VerifyResult> = Vec::new();
    let mut updated_lock = lock.clone();
    let claude_dir = get_claude_dir()?;
    
    for package in &lock.package {
        // Filter by skill name if specified
        if let Some(ref filter) = options.skill {
            if &package.name != filter {
                continue;
            }
        }
        
        // Determine skill path
        let skill_path = match package.scope {
            InstallScope::Global => {
                if package.install_path.is_empty() {
                    claude_dir.join("skills").join(&package.name).join("SKILL.md")
                } else {
                    PathBuf::from(&package.install_path).join("SKILL.md")
                }
            }
            InstallScope::Project => {
                project_root.join(".claude").join("skills").join(&package.name).join("SKILL.md")
            }
        };
        
        // Check if file exists
        if !skill_path.exists() {
            results.push(VerifyResult {
                skill_name: package.name.clone(),
                path: skill_path,
                expected_hash: package.content_hash.clone(),
                actual_hash: String::new(),
                is_valid: false,
                is_missing: true,
            });
            continue;
        }
        
        // Calculate current hash
        let actual_hash = hash_file(&skill_path)?;
        
        // Compare with recorded hash
        let is_valid = package.content_hash.as_ref().map(|h| h == &actual_hash).unwrap_or(true);
        
        results.push(VerifyResult {
            skill_name: package.name.clone(),
            path: skill_path.clone(),
            expected_hash: package.content_hash.clone(),
            actual_hash: actual_hash.clone(),
            is_valid,
            is_missing: false,
        });
        
        // Update hash if requested
        if options.update {
            if let Some(pkg) = updated_lock.package.iter_mut().find(|p| p.name == package.name) {
                pkg.content_hash = Some(actual_hash);
            }
        }
    }
    
    // Print results
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut missing_count = 0;
    let mut no_hash_count = 0;
    
    for result in &results {
        if result.is_missing {
            missing_count += 1;
            println!("❌ {} - MISSING", result.skill_name);
            println!("   Expected at: {}", result.path.display());
        } else if result.expected_hash.is_none() {
            no_hash_count += 1;
            println!("⚠️  {} - No checksum recorded", result.skill_name);
            if options.verbose {
                println!("   Current hash: {}", result.actual_hash);
            }
        } else if result.is_valid {
            valid_count += 1;
            println!("✅ {} - Verified", result.skill_name);
            if options.verbose {
                println!("   Hash: {}", result.actual_hash);
            }
        } else {
            invalid_count += 1;
            println!("🔴 {} - MODIFIED", result.skill_name);
            println!("   Expected: {}", result.expected_hash.as_ref().unwrap());
            println!("   Actual:   {}", result.actual_hash);
            println!("   ⚠️  File has been modified since installation!");
        }
    }
    
    // Summary
    println!("\n────────────────");
    println!("Summary:");
    println!("  ✅ Verified:    {}", valid_count);
    if invalid_count > 0 {
        println!("  🔴 Modified:    {}", invalid_count);
    }
    if missing_count > 0 {
        println!("  ❌ Missing:     {}", missing_count);
    }
    if no_hash_count > 0 {
        println!("  ⚠️  No checksum: {}", no_hash_count);
    }
    
    // Save updated lockfile if requested
    if options.update {
        updated_lock.save_to_project(&project_root)?;
        println!("\n📝 Updated checksums in Skills.lock");
    }
    
    // Exit with error if issues found
    if invalid_count > 0 || missing_count > 0 {
        println!("\n⚠️  Verification failed. Some skills may have been tampered with.");
        if !options.update {
            println!("   Use 'cowork verify --update' to update checksums if changes are intentional.");
        }
        std::process::exit(1);
    }
    
    Ok(())
}

/// Verify a single file against expected hash
#[allow(dead_code)]
pub fn verify_file(path: &Path, expected_hash: &str) -> Result<bool> {
    let actual_hash = hash_file(path)?;
    Ok(actual_hash == expected_hash)
}

/// Generate checksum for a skill and update lockfile
#[allow(dead_code)]
pub fn update_skill_checksum(project_root: &Path, skill_name: &str) -> Result<String> {
    let mut lock = SkillsLock::load_from_project(project_root)?
        .unwrap_or_default();
    
    let claude_dir = get_claude_dir()?;
    
    // Find skill path
    let skill_path = claude_dir.join("skills").join(skill_name).join("SKILL.md");
    if !skill_path.exists() {
        anyhow::bail!("Skill not found: {}", skill_name);
    }
    
    // Calculate hash
    let hash = hash_file(&skill_path)?;
    
    // Update lockfile
    if let Some(pkg) = lock.package.iter_mut().find(|p| p.name == skill_name) {
        pkg.content_hash = Some(hash.clone());
        pkg.touch();
    }
    
    lock.save_to_project(project_root)?;
    
    Ok(hash)
}
