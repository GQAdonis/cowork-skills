use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AgentConfig {
    pub name: &'static str,
    pub display_name: &'static str,
    pub skills_dir: &'static str,
    pub global_skills_dir: PathBuf,
}

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"))
}

pub fn get_all_agents() -> HashMap<&'static str, AgentConfig> {
    let home = get_home_dir();

    let mut agents = HashMap::new();

    agents.insert(
        "amp",
        AgentConfig {
            name: "amp",
            display_name: "Amp",
            skills_dir: ".agents/skills",
            global_skills_dir: home.join(".config/agents/skills"),
        },
    );

    agents.insert(
        "antigravity",
        AgentConfig {
            name: "antigravity",
            display_name: "Antigravity",
            skills_dir: ".agent/skills",
            global_skills_dir: home.join(".gemini/antigravity/skills"),
        },
    );

    agents.insert(
        "claude-code",
        AgentConfig {
            name: "claude-code",
            display_name: "Claude Code",
            skills_dir: ".claude/skills",
            global_skills_dir: home.join(".claude/skills"),
        },
    );

    agents.insert(
        "clawdbot",
        AgentConfig {
            name: "clawdbot",
            display_name: "Clawdbot",
            skills_dir: "skills",
            global_skills_dir: home.join(".clawdbot/skills"),
        },
    );

    agents.insert(
        "codex",
        AgentConfig {
            name: "codex",
            display_name: "Codex",
            skills_dir: ".codex/skills",
            global_skills_dir: home.join(".codex/skills"),
        },
    );

    agents.insert(
        "cursor",
        AgentConfig {
            name: "cursor",
            display_name: "Cursor",
            skills_dir: ".cursor/skills",
            global_skills_dir: home.join(".cursor/skills"),
        },
    );

    agents.insert(
        "droid",
        AgentConfig {
            name: "droid",
            display_name: "Droid",
            skills_dir: ".factory/skills",
            global_skills_dir: home.join(".factory/skills"),
        },
    );

    agents.insert(
        "gemini-cli",
        AgentConfig {
            name: "gemini-cli",
            display_name: "Gemini CLI",
            skills_dir: ".gemini/skills",
            global_skills_dir: home.join(".gemini/skills"),
        },
    );

    agents.insert(
        "github-copilot",
        AgentConfig {
            name: "github-copilot",
            display_name: "GitHub Copilot",
            skills_dir: ".github/skills",
            global_skills_dir: home.join(".copilot/skills"),
        },
    );

    agents.insert(
        "goose",
        AgentConfig {
            name: "goose",
            display_name: "Goose",
            skills_dir: ".goose/skills",
            global_skills_dir: home.join(".config/goose/skills"),
        },
    );

    agents.insert(
        "kilo",
        AgentConfig {
            name: "kilo",
            display_name: "Kilo Code",
            skills_dir: ".kilocode/skills",
            global_skills_dir: home.join(".kilocode/skills"),
        },
    );

    agents.insert(
        "kiro-cli",
        AgentConfig {
            name: "kiro-cli",
            display_name: "Kiro CLI",
            skills_dir: ".kiro/skills",
            global_skills_dir: home.join(".kiro/skills"),
        },
    );

    agents.insert(
        "opencode",
        AgentConfig {
            name: "opencode",
            display_name: "OpenCode",
            skills_dir: ".opencode/skills",
            global_skills_dir: home.join(".config/opencode/skills"),
        },
    );

    agents.insert(
        "roo",
        AgentConfig {
            name: "roo",
            display_name: "Roo Code",
            skills_dir: ".roo/skills",
            global_skills_dir: home.join(".roo/skills"),
        },
    );

    agents.insert(
        "trae",
        AgentConfig {
            name: "trae",
            display_name: "Trae",
            skills_dir: ".trae/skills",
            global_skills_dir: home.join(".trae/skills"),
        },
    );

    agents.insert(
        "windsurf",
        AgentConfig {
            name: "windsurf",
            display_name: "Windsurf",
            skills_dir: ".windsurf/skills",
            global_skills_dir: home.join(".codeium/windsurf/skills"),
        },
    );

    // Zed: primary path is ~/.config/zed/skills/, fallback is ~/.zed/skills/
    // Skills are plain directory drops — no manifest or plugin API required.
    let zed_skills_dir = if home.join(".config/zed").exists() {
        home.join(".config/zed/skills")
    } else {
        home.join(".zed/skills")
    };
    agents.insert(
        "zed",
        AgentConfig {
            name: "zed",
            display_name: "Zed",
            skills_dir: ".config/zed/skills",
            global_skills_dir: zed_skills_dir,
        },
    );

    agents
}

pub fn detect_installed_agents() -> Vec<&'static str> {
    let home = get_home_dir();
    let mut installed = Vec::new();

    let checks = [
        ("amp", home.join(".config/amp")),
        ("antigravity", home.join(".gemini/antigravity")),
        ("claude-code", home.join(".claude")),
        ("clawdbot", home.join(".clawdbot")),
        ("codex", home.join(".codex")),
        ("cursor", home.join(".cursor")),
        ("droid", home.join(".factory")),
        ("gemini-cli", home.join(".gemini")),
        ("github-copilot", home.join(".copilot")),
        ("goose", home.join(".config/goose")),
        ("kilo", home.join(".kilocode")),
        ("kiro-cli", home.join(".kiro")),
        ("opencode", home.join(".config/opencode")),
        ("roo", home.join(".roo")),
        ("trae", home.join(".trae")),
        ("windsurf", home.join(".codeium/windsurf")),
        ("zed", home.join(".config/zed")),
    ];

    for (name, path) in checks {
        if path.exists() {
            installed.push(name);
        }
    }

    // Zed has a dual install location: ~/.config/zed/ (XDG) or ~/.zed/ (legacy)
    if !installed.contains(&"zed")
        && (home.join(".zed").exists())
    {
        installed.push("zed");
    }

    installed
}

pub fn get_agent_names() -> Vec<&'static str> {
    vec![
        "amp",
        "antigravity",
        "claude-code",
        "clawdbot",
        "codex",
        "cursor",
        "droid",
        "gemini-cli",
        "github-copilot",
        "goose",
        "kilo",
        "kiro-cli",
        "opencode",
        "roo",
        "trae",
        "windsurf",
        "zed",
    ]
}
