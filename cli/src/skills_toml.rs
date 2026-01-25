//! `Skills.toml` configuration format for managing project skills.
//!
//! Similar to Cargo.toml but for Claude Code skills.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The directory for cowork configuration
pub const COWORK_DIR: &str = ".cowork";

/// The filename for skills configuration
pub const SKILLS_TOML: &str = "Skills.toml";

/// The filename for skills lock file
pub const SKILLS_LOCK: &str = "Skills.lock";

/// Get the full path to Skills.toml in a project
pub fn skills_toml_path(project_root: &Path) -> PathBuf {
    project_root.join(COWORK_DIR).join(SKILLS_TOML)
}

/// Get the full path to Skills.lock in a project
pub fn skills_lock_path(project_root: &Path) -> PathBuf {
    project_root.join(COWORK_DIR).join(SKILLS_LOCK)
}

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsToml {
    /// Project metadata
    #[serde(default)]
    pub project: ProjectSection,

    /// Skills configuration
    #[serde(default)]
    pub skills: SkillsSection,

    /// Trigger configuration
    #[serde(default)]
    pub triggers: TriggersSection,

    /// Security configuration
    #[serde(default)]
    pub security: crate::security::SecurityConfig,
}

/// Project metadata section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSection {
    /// Project name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Project description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Skills configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsSection {
    /// Global skills configuration (~/.claude/skills/)
    #[serde(default)]
    pub global: SkillSourceConfig,

    /// GitHub dependencies (like Cargo)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub install: HashMap<String, SkillDependency>,

    /// Local development links (symlinks for testing)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dev: HashMap<String, DevLink>,

    /// Skill groups configuration
    #[serde(default)]
    pub groups: GroupsConfig,
}

/// Configuration for a skill source (global or local)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillSourceConfig {
    /// Enabled skills (whitelist - if set, only these are loaded)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled: Vec<String>,

    /// Disabled skills (blacklist - excluded from loading)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled: Vec<String>,
}

/// Skill dependency (similar to Cargo dependency)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillDependency {
    /// Simple form: "user/repo"
    Simple(String),

    /// Detailed form with options
    Detailed(SkillDependencyDetail),
}

/// Detailed skill dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillDependencyDetail {
    /// GitHub repository (user/repo)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,

    /// Local path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Specific skills to install (if empty, install all)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,

    /// Git reference (branch, tag, commit)
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,

    /// Target agents
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<String>,

    /// Install as plugin (preserves full repository structure)
    #[serde(default, skip_serializing_if = "is_false")]
    pub plugin: bool,

    /// Install to project local (.claude/skills/) instead of global
    #[serde(default, skip_serializing_if = "is_false")]
    pub local: bool,

    /// Enable/disable this dependency (default: true)
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_true(b: &bool) -> bool {
    *b
}

/// Development link specification (symlink for testing)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DevLink {
    /// Simple form: just the path (defaults to local)
    Simple(String),

    /// Detailed form with options
    Detailed(DevLinkDetail),
}

/// Detailed development link specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevLinkDetail {
    /// Local path to link
    pub path: String,

    /// Link to project local (default: true) or global
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub local: bool,

    /// Enable/disable this link (default: true)
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub enabled: bool,

    /// Link as plugin (to .claude/<name>/) instead of skill (to .claude/skills/<name>/)
    #[serde(default, skip_serializing_if = "is_false")]
    pub plugin: bool,
}

impl DevLink {
    /// Get the path
    pub fn path(&self) -> &str {
        match self {
            DevLink::Simple(s) => s,
            DevLink::Detailed(d) => &d.path,
        }
    }

    /// Check if this should be linked to project local (default: true)
    pub fn is_local(&self) -> bool {
        match self {
            DevLink::Simple(_) => true, // default to local for dev
            DevLink::Detailed(d) => d.local,
        }
    }

    /// Check if this link is enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            DevLink::Simple(_) => true,
            DevLink::Detailed(d) => d.enabled,
        }
    }

    /// Check if this should be linked as a plugin
    pub fn is_plugin(&self) -> bool {
        match self {
            DevLink::Simple(_) => false,
            DevLink::Detailed(d) => d.plugin,
        }
    }
}

/// Skill groups configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupsConfig {
    /// Enabled skill groups
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled: Vec<String>,

    /// Disabled skill groups
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled: Vec<String>,
}

/// Triggers configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TriggersSection {
    /// Priority order for skills (first = highest priority)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub priority: Vec<String>,

    /// Explicit trigger -> skill overrides
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub overrides: HashMap<String, String>,
}

impl SkillsToml {
    /// Load from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Load from project root (looks for .cowork/skills.toml)
    pub fn load_from_project(project_root: &Path) -> Result<Option<Self>> {
        let path = skills_toml_path(project_root);
        if path.exists() {
            Ok(Some(Self::load(&path)?))
        } else {
            Ok(None)
        }
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize Skills.toml")?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))
    }

    /// Save to project root (saves to .cowork/skills.toml)
    #[allow(dead_code)]
    pub fn save_to_project(&self, project_root: &Path) -> Result<PathBuf> {
        let cowork_dir = project_root.join(COWORK_DIR);
        if !cowork_dir.exists() {
            fs::create_dir_all(&cowork_dir)
                .with_context(|| format!("Failed to create {}", cowork_dir.display()))?;
        }
        let path = skills_toml_path(project_root);
        self.save(&path)?;
        Ok(path)
    }

    /// Create a default configuration
    #[allow(dead_code)]
    pub fn default_config(project_name: Option<&str>) -> Self {
        Self {
            project: ProjectSection {
                name: project_name.map(String::from),
                description: None,
            },
            skills: SkillsSection {
                global: SkillSourceConfig {
                    enabled: vec![
                        "memory-filesystem".to_string(),
                        "best-skill-creator".to_string(),
                    ],
                    disabled: vec![],
                },
                install: HashMap::new(),
                dev: HashMap::new(),
                groups: GroupsConfig {
                    enabled: vec!["rust-core".to_string()],
                    disabled: vec![],
                },
            },
            triggers: TriggersSection {
                priority: vec![],
                overrides: HashMap::new(),
            },
            security: crate::security::SecurityConfig::default(),
        }
    }

    /// Create an example configuration with comments
    pub fn example_toml() -> &'static str {
        r#"# Skills.toml - Project skill configuration
# Similar to Cargo.toml but for Claude Code skills

[project]
name = "my-project"
# description = "My awesome project"

# ============================================================
# Skill Sources
# ============================================================

[skills.global]
# Global skills from ~/.claude/skills/
# If 'enabled' is set, only these skills are loaded (whitelist)
# If 'disabled' is set, these skills are excluded (blacklist)
enabled = [
    "memory-filesystem",
    "best-skill-creator",
]
disabled = []

# ============================================================
# Skill Dependencies (install from GitHub or local)
# ============================================================

[skills.install]
# Simple form: name = "user/repo" (installs to global)
# rust-skills = "ZhangHanDong/rust-skills"

# Detailed form with options:
# tokio = { repo = "user/tokio-skills", skills = ["tokio-runtime", "tokio-sync"] }
# pinned = { repo = "user/repo", ref = "v1.0.0" }

# Install to project local instead of global:
# my-skills = { repo = "user/skills", local = true }

# Plugin form (preserves full repository structure):
# makepad = { repo = "user/makepad-skills", plugin = true }
# dora = { repo = "user/dora-skills", plugin = true, local = true }

# Disabled dependency (installed but not enabled):
# old-lib = { repo = "user/old", enabled = false }

# ============================================================
# Development Links (symlinks for testing local skills)
# ============================================================

[skills.dev]
# Simple form: links to .claude/skills/ (project local by default)
# my-skill = "/path/to/my-skill-project"

# Detailed form:
# dora-dev = { path = "/path/to/dora-skills" }
# global-test = { path = "/path/to/skills", local = false }

# Plugin form: links to .claude/<name>/ instead of .claude/skills/
# dora-plugin = { path = "/path/to/dora-skills", plugin = true }

# ============================================================
# Skill Groups (batch enable/disable)
# ============================================================

[skills.groups]
# Available groups:
#   rust-core     - Basic Rust skills (ownership, concurrency, etc.)
#   rust-patterns - Design patterns (domain modeling, performance, etc.)
#   rust-domains  - Domain-specific (web, CLI, fintech, etc.)
#   makepad       - Makepad UI framework
#   dora          - Dora-rs robotics
enabled = ["rust-core"]
disabled = []

# ============================================================
# Trigger Conflict Resolution
# ============================================================

[triggers]
# Priority order for skills when triggers conflict
# First skill in list has highest priority
priority = [
    # "dora-router",
    # "rust-router",
    # "cowork-router",
]

[triggers.overrides]
# Explicitly map triggers to specific skills
# "async" = "rust-router"
# "widget" = "makepad-router"
# "node" = "dora-router"

# ============================================================
# Security Configuration
# ============================================================

[security]
# Trusted GitHub authors/organizations (skills from these sources are trusted)
trusted_authors = [
    # "ZhangHanDong",
    # "anthropics",
]

# Additional blocked patterns to detect in SKILL.md files
# (extends built-in patterns like "rm -rf", "curl|sh", etc.)
blocked_patterns = [
    # "custom-dangerous-pattern",
]

# Paths to skip during security scanning (glob patterns)
# Use this to exclude documentation or known-safe files from audit
skip_paths = [
    # "**/hookify/skills/writing-rules/**",  # Documentation about security patterns
    # "**/docs/**",                           # General documentation
    # "**/examples/**",                       # Example code
]

# Trusted marketplace plugin prefixes (skip scanning these plugins)
# Plugins from these marketplaces are trusted and won't be scanned
trusted_marketplaces = [
    # "hookify",
    # "claude-plugins-official",
]

# Require lockfile for all installations
require_lockfile = false

# Verify checksums when loading skills
verify_checksums = false

# Show diff before updating skills
show_update_diff = true

# Automatically reject skills with HIGH risk issues
auto_reject_high_risk = false
"#
    }

    /// Get all skills that should be enabled (resolved from config)
    #[allow(dead_code)]
    pub fn get_enabled_skills(&self) -> EnabledSkills {
        EnabledSkills {
            global: self.skills.global.enabled.clone(),
            groups: self.skills.groups.enabled.clone(),
        }
    }

    /// Get all skills that should be disabled
    #[allow(dead_code)]
    pub fn get_disabled_skills(&self) -> DisabledSkills {
        DisabledSkills {
            global: self.skills.global.disabled.clone(),
            groups: self.skills.groups.disabled.clone(),
        }
    }

    /// Get install dependencies
    #[allow(dead_code)]
    pub fn get_dependencies(&self) -> &HashMap<String, SkillDependency> {
        &self.skills.install
    }
}

/// Resolved enabled skills
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct EnabledSkills {
    pub global: Vec<String>,
    pub groups: Vec<String>,
}

/// Resolved disabled skills
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct DisabledSkills {
    pub global: Vec<String>,
    pub groups: Vec<String>,
}

impl SkillDependency {
    /// Get the repository string
    pub fn repo(&self) -> Option<&str> {
        match self {
            SkillDependency::Simple(s) => Some(s),
            SkillDependency::Detailed(d) => d.repo.as_deref(),
        }
    }

    /// Get the local path
    pub fn path(&self) -> Option<&str> {
        match self {
            SkillDependency::Simple(_) => None,
            SkillDependency::Detailed(d) => d.path.as_deref(),
        }
    }

    /// Get specific skills to install
    pub fn skills(&self) -> &[String] {
        match self {
            SkillDependency::Simple(_) => &[],
            SkillDependency::Detailed(d) => &d.skills,
        }
    }

    /// Get git reference
    pub fn git_ref(&self) -> Option<&str> {
        match self {
            SkillDependency::Simple(_) => None,
            SkillDependency::Detailed(d) => d.git_ref.as_deref(),
        }
    }

    /// Check if this should be installed as a plugin
    pub fn is_plugin(&self) -> bool {
        match self {
            SkillDependency::Simple(_) => false,
            SkillDependency::Detailed(d) => d.plugin,
        }
    }

    /// Get target agents
    pub fn agents(&self) -> &[String] {
        match self {
            SkillDependency::Simple(_) => &[],
            SkillDependency::Detailed(d) => &d.agents,
        }
    }

    /// Check if this should be installed to project local
    pub fn is_local(&self) -> bool {
        match self {
            SkillDependency::Simple(_) => false,
            SkillDependency::Detailed(d) => d.local,
        }
    }

    /// Check if this dependency is enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            SkillDependency::Simple(_) => true,
            SkillDependency::Detailed(d) => d.enabled,
        }
    }
}

/// Predefined skill groups
pub fn get_skill_groups() -> HashMap<&'static str, Vec<&'static str>> {
    let mut groups = HashMap::new();

    groups.insert("rust-core", vec![
        "rust-router",
        "m01-ownership",
        "m02-resource",
        "m03-mutability",
        "m04-zero-cost",
        "m05-type-driven",
        "m06-error-handling",
        "m07-concurrency",
    ]);

    groups.insert("rust-patterns", vec![
        "m09-domain",
        "m10-performance",
        "m11-ecosystem",
        "m12-lifecycle",
        "m13-domain-error",
        "m14-mental-model",
        "m15-anti-pattern",
    ]);

    groups.insert("rust-domains", vec![
        "domain-cli",
        "domain-web",
        "domain-fintech",
        "domain-embedded",
        "domain-iot",
        "domain-ml",
        "domain-cloud-native",
    ]);

    groups.insert("makepad", vec![
        "makepad-router",
        "makepad-basics",
        "makepad-dsl",
        "makepad-widgets",
        "makepad-layout",
        "makepad-event-action",
        "makepad-shaders",
        "makepad-animation",
        "makepad-font",
        "makepad-platform",
        "makepad-splash",
    ]);

    groups.insert("dora", vec![
        "dora-router",
        "node-api-rust",
        "node-api-python",
        "operator-api",
        "dataflow-config",
        "data-pipeline",
        "integration-testing",
        "cli-commands",
    ]);

    groups.insert("dora-hubs", vec![
        "hub-audio",
        "hub-camera",
        "hub-detection",
        "hub-llm",
        "hub-nodes",
        "hub-recording",
        "hub-robot",
        "hub-translation",
        "hub-visualization",
    ]);

    groups
}

// ============================================================
// Skills.lock - Lockfile for installed skills
// ============================================================

/// The lock file structure for tracking installed skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsLock {
    /// Lock file version
    #[serde(default = "default_lock_version")]
    pub version: u32,

    /// Installed packages (skills and plugins)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub package: Vec<LockedPackage>,
}

impl Default for SkillsLock {
    fn default() -> Self {
        Self {
            version: 1,
            package: Vec::new(),
        }
    }
}

fn default_lock_version() -> u32 {
    1
}

/// A locked (installed) package entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Package name
    pub name: String,

    /// Package version (from plugin.json, package.json, or install timestamp)
    pub version: String,

    /// Source type
    pub source: PackageSource,

    /// Installation scope
    pub scope: InstallScope,

    /// Installation type
    #[serde(rename = "type")]
    pub install_type: InstallType,

    /// Installation path
    pub install_path: String,

    /// Source path or repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,

    /// Git commit SHA (if from git)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<String>,

    /// Git reference (branch/tag) if specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,

    /// Installation timestamp
    pub installed_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Whether currently enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// List of installed skills (for skill packages)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,

    /// Content hash for integrity verification (SHA-256)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

/// Package source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PackageSource {
    /// From GitHub repository
    Github,
    /// From local path
    Local,
    /// Development symlink
    Dev,
}

/// Installation scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstallScope {
    /// Global installation (~/.claude/)
    Global,
    /// Project-local installation (.claude/)
    Project,
}

/// Installation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstallType {
    /// Plugin (full repository structure)
    Plugin,
    /// Individual skills
    Skills,
    /// Symlink for development
    Symlink,
}

impl SkillsLock {
    /// Load from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Load from project root
    pub fn load_from_project(project_root: &Path) -> Result<Option<Self>> {
        let path = skills_lock_path(project_root);
        if path.exists() {
            Ok(Some(Self::load(&path)?))
        } else {
            Ok(None)
        }
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize Skills.lock")?;

        // Add header comment
        let header = "# This file is auto-generated by cowork. Do not edit manually.\n\n";
        let full_content = format!("{}{}", header, content);

        fs::write(path, full_content)
            .with_context(|| format!("Failed to write {}", path.display()))
    }

    /// Save to project root
    pub fn save_to_project(&self, project_root: &Path) -> Result<PathBuf> {
        let cowork_dir = project_root.join(COWORK_DIR);
        if !cowork_dir.exists() {
            fs::create_dir_all(&cowork_dir)
                .with_context(|| format!("Failed to create {}", cowork_dir.display()))?;
        }
        let path = skills_lock_path(project_root);
        self.save(&path)?;
        Ok(path)
    }

    /// Add or update a package entry
    pub fn upsert_package(&mut self, package: LockedPackage) {
        // Remove existing entry with same name
        self.package.retain(|p| p.name != package.name);
        self.package.push(package);
        // Sort by name for consistent output
        self.package.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// Remove a package by name
    #[allow(dead_code)]
    pub fn remove_package(&mut self, name: &str) -> bool {
        let len_before = self.package.len();
        self.package.retain(|p| p.name != name);
        self.package.len() < len_before
    }

    /// Get a package by name
    #[allow(dead_code)]
    pub fn get_package(&self, name: &str) -> Option<&LockedPackage> {
        self.package.iter().find(|p| p.name == name)
    }

    /// Check if a package is installed
    #[allow(dead_code)]
    pub fn is_installed(&self, name: &str) -> bool {
        self.package.iter().any(|p| p.name == name)
    }
}

impl LockedPackage {
    /// Create a new package entry with timestamp-based version
    pub fn new_with_timestamp(name: &str) -> Self {
        let now = Utc::now();
        Self {
            name: name.to_string(),
            version: now.format("%Y%m%d.%H%M%S").to_string(),
            source: PackageSource::Local,
            scope: InstallScope::Project,
            install_type: InstallType::Skills,
            install_path: String::new(),
            source_path: None,
            git_sha: None,
            git_ref: None,
            installed_at: now,
            updated_at: now,
            enabled: true,
            skills: Vec::new(),
            content_hash: None,
        }
    }

    /// Create a package entry from a GitHub source
    pub fn from_github(
        name: &str,
        repo: &str,
        version: Option<String>,
        git_sha: Option<String>,
        git_ref: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            name: name.to_string(),
            version: version.unwrap_or_else(|| now.format("%Y%m%d.%H%M%S").to_string()),
            source: PackageSource::Github,
            scope: InstallScope::Global,
            install_type: InstallType::Skills,
            install_path: String::new(),
            source_path: Some(repo.to_string()),
            git_sha,
            git_ref,
            installed_at: now,
            updated_at: now,
            enabled: true,
            skills: Vec::new(),
            content_hash: None,
        }
    }

    /// Create a dev link package entry
    pub fn from_dev_link(name: &str, source_path: &str, install_path: &str, is_local: bool) -> Self {
        let now = Utc::now();
        Self {
            name: name.to_string(),
            version: now.format("%Y%m%d.%H%M%S").to_string(),
            source: PackageSource::Dev,
            scope: if is_local { InstallScope::Project } else { InstallScope::Global },
            install_type: InstallType::Symlink,
            install_path: install_path.to_string(),
            source_path: Some(source_path.to_string()),
            git_sha: None,
            git_ref: None,
            installed_at: now,
            updated_at: now,
            enabled: true,
            skills: Vec::new(),
            content_hash: None,
        }
    }

    /// Set as plugin type
    #[allow(dead_code)]
    pub fn as_plugin(mut self) -> Self {
        self.install_type = InstallType::Plugin;
        self
    }

    /// Set scope
    #[allow(dead_code)]
    pub fn with_scope(mut self, scope: InstallScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set install path
    #[allow(dead_code)]
    pub fn with_install_path(mut self, path: &str) -> Self {
        self.install_path = path.to_string();
        self
    }

    /// Set installed skills
    #[allow(dead_code)]
    pub fn with_skills(mut self, skills: Vec<String>) -> Self {
        self.skills = skills;
        self
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Set content hash
    #[allow(dead_code)]
    pub fn with_content_hash(mut self, hash: String) -> Self {
        self.content_hash = Some(hash);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dependency() {
        let toml_str = r#"
[skills.install]
rust-skills = "ZhangHanDong/rust-skills"
"#;
        let config: SkillsToml = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.skills.install.get("rust-skills").unwrap().repo(),
            Some("ZhangHanDong/rust-skills")
        );
    }

    #[test]
    fn test_parse_detailed_dependency() {
        let toml_str = r#"
[skills.install]
tokio = { repo = "user/tokio-skills", skills = ["tokio-runtime"], ref = "v1.0" }
"#;
        let config: SkillsToml = toml::from_str(toml_str).unwrap();
        let dep = config.skills.install.get("tokio").unwrap();
        assert_eq!(dep.repo(), Some("user/tokio-skills"));
        assert_eq!(dep.skills(), &["tokio-runtime"]);
        assert_eq!(dep.git_ref(), Some("v1.0"));
    }

    #[test]
    fn test_parse_path_dependency() {
        let toml_str = r#"
[skills.install]
local = { path = "../my-skills" }
"#;
        let config: SkillsToml = toml::from_str(toml_str).unwrap();
        let dep = config.skills.install.get("local").unwrap();
        assert_eq!(dep.path(), Some("../my-skills"));
    }
}
