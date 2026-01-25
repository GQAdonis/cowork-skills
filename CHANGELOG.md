# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-26

### Added

- **CLI Tool** (`cowork` / `co`)
  - `cowork init` - Install built-in skills to `~/.claude/skills/`
  - `cowork install` - Install skills from GitHub repositories
  - `cowork generate` - Generate skills from source code (Rust, TypeScript, Python)
  - `cowork search` - Search GitHub for skill repositories
  - `cowork config` - Manage project-level configuration via `Skills.toml`
  - `cowork plugins` - Manage Claude Code marketplace plugins
  - `cowork list` - List all available skills
  - `cowork status` - Show current configuration
  - `cowork doctor` - Check for configuration issues
  - `cowork test` - Generate and run trigger tests
  - `cowork audit` - Security audit of installed skills
  - `cowork verify` - Verify checksums against `Skills.lock`

- **Built-in Skills**
  - `memory-skills` - CoALA cognitive architecture memory system
  - `cowork-guide` - Complete CLI usage guide
  - `cowork-router` - Unified router for installed plugins/skills
  - `code-review` - Code review assistant with best practices
  - `github-generate` - Generate skills from GitHub repositories
  - `github-search` - Search GitHub for skill repositories

- **Multi-Agent Support**
  - Install skills to 16+ AI coding agents
  - Supported: Claude Code, Cursor, Codex, GitHub Copilot, Windsurf, Goose, Amp, Roo, Kiro CLI, Gemini CLI, and more

- **Source Code Parsing**
  - Rust parser using `syn` crate
  - TypeScript parser using `tree-sitter`
  - Python parser using `tree-sitter`

- **Project Configuration**
  - `Skills.toml` for dependency management
  - `Skills.lock` for checksum verification
  - Skill groups and trigger priority configuration

- **Security Features**
  - Dangerous pattern detection (`rm -rf`, `eval()`, `curl|sh`)
  - Prompt injection detection
  - Credential leak detection
  - Risk level classification (SAFE, LOW, MEDIUM, HIGH, CRITICAL)

- **One-line Installer**
  - `curl -sSL https://raw.githubusercontent.com/ZhangHanDong/cowork-skills/main/install.sh | bash`

- **Documentation**
  - Trilingual README (English, Chinese, Japanese)
  - Comprehensive CLI usage guide

[Unreleased]: https://github.com/ZhangHanDong/cowork-skills/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ZhangHanDong/cowork-skills/releases/tag/v0.1.0
