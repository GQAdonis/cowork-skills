use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Legacy config file (deprecated, use Skills.toml instead)
#[allow(dead_code)]
const CONFIG_FILE: &str = "cowork-config.json";

/// Legacy config structure (deprecated)
#[derive(Debug, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct Config {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub skill_sources: SkillSources,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub domain_keywords: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct SkillSources {
    #[serde(default)]
    pub project: ProjectSkills,
    #[serde(default)]
    pub global: GlobalSkills,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct ProjectSkills {
    #[serde(default)]
    pub plugins: HashMap<String, String>,
    #[serde(default)]
    pub external: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct GlobalSkills {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub referenced: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct RoutingConfig {
    #[serde(default)]
    pub entry_point: String,
    #[serde(default)]
    pub cross_domain_enabled: bool,
    #[serde(default)]
    pub inherit_rust_layers: bool,
}

impl Config {
    #[allow(dead_code)]
    pub fn load(cowork_root: &Path) -> Result<Self> {
        let config_path = cowork_root.join(CONFIG_FILE);

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            serde_json::from_str(&content).context("Failed to parse config file")
        } else {
            Ok(Self::default_config())
        }
    }

    #[allow(dead_code)]
    pub fn save(&self, cowork_root: &Path) -> Result<()> {
        let config_path = cowork_root.join(CONFIG_FILE);
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, content).context("Failed to write config file")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn default_config() -> Self {
        let mut domain_keywords = HashMap::new();
        domain_keywords.insert(
            "makepad".to_string(),
            vec![
                "makepad".to_string(),
                "widget".to_string(),
                "view".to_string(),
                "live_design".to_string(),
                "Draw2d".to_string(),
                "WidgetRef".to_string(),
                "robius".to_string(),
            ],
        );
        domain_keywords.insert(
            "dora".to_string(),
            vec![
                "dora".to_string(),
                "node".to_string(),
                "operator".to_string(),
                "dataflow".to_string(),
                "robot".to_string(),
                "sensor".to_string(),
                "DoraNode".to_string(),
            ],
        );
        domain_keywords.insert(
            "rust".to_string(),
            vec![
                r"E0\d{3,4}".to_string(),
                "ownership".to_string(),
                "borrow".to_string(),
                "lifetime".to_string(),
                "async".to_string(),
                "trait".to_string(),
                "unsafe".to_string(),
            ],
        );

        Self {
            version: "1.0.0".to_string(),
            skill_sources: SkillSources {
                project: ProjectSkills {
                    plugins: HashMap::new(),
                    external: HashMap::new(),
                },
                global: GlobalSkills {
                    path: "~/.claude/skills".to_string(),
                    referenced: vec![
                        "memory-filesystem".to_string(),
                        "best-skill-creator".to_string(),
                        "tokio-*".to_string(),
                        "ratatui-*".to_string(),
                        "os-checker-*".to_string(),
                    ],
                },
            },
            routing: RoutingConfig {
                entry_point: "cowork-router".to_string(),
                cross_domain_enabled: true,
                inherit_rust_layers: true,
            },
            domain_keywords,
        }
    }

    #[allow(dead_code)]
    pub fn add_external_skill(&mut self, name: &str, path: &str) {
        self.skill_sources.project.external.insert(
            name.to_string(),
            path.to_string(),
        );
    }

    #[allow(dead_code)]
    pub fn remove_external_skill(&mut self, name: &str) -> bool {
        self.skill_sources.project.external.remove(name).is_some()
    }
}

/// Get the Claude configuration directory (~/.claude/)
pub fn get_claude_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".claude"))
}

/// Get the global skills directory (~/.claude/skills/)
#[allow(dead_code)]
pub fn get_global_skills_dir() -> Result<PathBuf> {
    Ok(get_claude_dir()?.join("skills"))
}

/// Get the cowork repos directory (~/.cowork/repos/)
#[allow(dead_code)]
pub fn get_repos_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".cowork").join("repos"))
}

/// Find the cowork-skills root directory by looking for .cowork/ or CLAUDE.md
pub fn find_cowork_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Walk up the directory tree to find the root
    let mut dir = current_dir.as_path();
    loop {
        // Check for indicators of cowork-skills root (new: .cowork/Skills.toml)
        if dir.join(".cowork").join("Skills.toml").exists()
            || dir.join(".cowork").exists()
            || (dir.join("CLAUDE.md").exists() && dir.join("skills").exists())
        {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // If not found, assume current directory is root
    Ok(current_dir)
}
