# CoWork Skills CLI

[![Crates.io](https://img.shields.io/crates/v/cowork.svg)](https://crates.io/crates/cowork)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

CLI tool for managing Claude Code skills - install, generate, and organize skills across AI coding agents.

## Installation

```bash
# From crates.io
cargo install cowork

# Or one-line installer
curl -sSL https://raw.githubusercontent.com/ZhangHanDong/cowork-skills/main/install.sh | bash
```

## Quick Start

```bash
# Initialize built-in skills
cowork init

# Install skills from GitHub
cowork install user/repo

# Search for skill repositories
cowork search agent-skill --topic

# List all available skills
cowork list

# Check configuration status
cowork status
```

## Commands

| Command | Description |
|---------|-------------|
| `cowork init` | Install built-in skills to `~/.claude/skills/` |
| `cowork install` | Install skills from GitHub repositories |
| `cowork generate` | Generate skills from source code (Rust, TypeScript, Python) |
| `cowork search` | Search GitHub for skill repositories |
| `cowork config` | Manage project-level configuration via `Skills.toml` |
| `cowork plugins` | Manage Claude Code marketplace plugins |
| `cowork list` | List all available skills |
| `cowork status` | Show current configuration |
| `cowork doctor` | Check for configuration issues |
| `cowork test` | Generate and run trigger tests |
| `cowork audit` | Security audit of installed skills |
| `cowork verify` | Verify checksums against `Skills.lock` |

## Built-in Skills

The CLI includes 12 built-in skills embedded at compile time:

| Skill | Description |
|-------|-------------|
| `best-skill-creator` | SKILL.md template and best practices |
| `code-review` | GitHub PR review integration |
| `cowork-guide` | Complete CLI usage guide |
| `cowork-router` | Unified router for plugins/skills |
| `github-generate` | Generate skills from GitHub repos |
| `github-search` | Search GitHub for skill repos |
| `memory-skills` | CoALA cognitive architecture memory |
| `recall` | Search and retrieve from memory |
| `remember` | Save information to memory |
| `summarize-session` | Summarize session to episodic memory |
| `remotion` | Video creation in React |
| `writing-assistant` | Writing assistance and grammar |

## Multi-Agent Support

Install skills to 16+ AI coding agents:

| Agent | Status |
|-------|--------|
| Claude Code | Fully Tested |
| Cursor, Codex, GitHub Copilot, Windsurf, Goose, Amp, Roo, Kiro CLI, Gemini CLI | Community Testing |

## Configuration

Use `Skills.toml` for project-level configuration:

```toml
[project]
name = "my-project"

[skills.install]
rust-skills = { repo = "user/rust-skills" }

[skills.global]
enabled = ["memory-skills", "code-review"]
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- [GitHub Repository](https://github.com/ZhangHanDong/cowork-skills)
- [Documentation](https://github.com/ZhangHanDong/cowork-skills#readme)
- [Changelog](https://github.com/ZhangHanDong/cowork-skills/blob/main/CHANGELOG.md)
