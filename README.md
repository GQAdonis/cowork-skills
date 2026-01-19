# CoWork Skills

[中文](./README-zh.md) | [日本語](./README-ja.md)

> Integrated Rust development assistant for multiple domains

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-Plugin-blue)](https://github.com/anthropics/claude-code)

## What is CoWork Skills?

**CoWork Skills** is a parent plugin that integrates multiple Rust domain skills into a unified development experience:

- **rust-skills** - Core Rust language knowledge (ownership, concurrency, error handling)
- **makepad-skills** - Makepad UI framework development
- **dora-skills** - Dora-rs robotics framework development

## Architecture

```
cowork-skills/
├── skills/
│   ├── cowork-router/     # Master router (native)
│   ├── rust-router/       # → symlink to plugins/rust-skills/skills/
│   ├── m01-ownership/     # → symlink to plugins/rust-skills/skills/
│   └── ...                # All sub-plugin skills via symlinks
├── plugins/
│   ├── rust-skills/       # Git submodule - Core Rust
│   ├── makepad-skills/    # Git submodule - UI
│   └── dora-skills/       # Git submodule - Robotics
├── .claude/hooks/         # Unified hooks
└── sync-skills.sh         # Script to sync symlinks
```

> **Note**: Claude Code only loads skills from the plugin's root `skills/` directory. We use symlinks to include sub-plugin skills.

## Installation

### Clone with Submodules

```bash
git clone --recurse-submodules https://github.com/ZhangHanDong/cowork-skills.git
```

### Launch Claude Code

```bash
claude --plugin-dir /path/to/cowork-skills
```

### Permission Configuration

Copy the example settings to your project:

```bash
cp /path/to/cowork-skills/.claude/settings.example.json .claude/settings.local.json
```

## How It Works

```
User Question
     │
     ▼
┌─────────────────────────────────┐
│       cowork-router-hook        │
│  Detect domain from keywords    │
└─────────────────────────────────┘
     │
     ├─────────────┬─────────────┐
     ▼             ▼             ▼
┌─────────┐  ┌─────────┐  ┌─────────┐
│ Makepad │  │  Dora   │  │  Rust   │
│ Router  │  │ Router  │  │ Router  │
└─────────┘  └─────────┘  └─────────┘
     │             │             │
     └─────────────┴─────────────┘
                   │
                   ▼
         Domain-aware Answer
```

## Domain Skills

| Domain | Plugin | Description |
|--------|--------|-------------|
| **Rust Core** | rust-skills | Ownership, concurrency, error handling, meta-cognition framework |
| **UI Development** | makepad-skills | Makepad widgets, views, live design |
| **Robotics** | dora-skills | Dora nodes, operators, dataflow |

## Cross-Domain Questions

CoWork Skills handles questions that span multiple domains:

```
User: "How to handle E0382 in Makepad widget?"

CoWork Router:
├── Primary: Makepad (UI context)
├── Secondary: E0382 (Rust ownership)
└── Action: Load both skills, combine knowledge
```

## Updating Submodules

```bash
cd cowork-skills

# Update all submodules
git submodule update --remote

# Re-sync skills symlinks after update
./sync-skills.sh

# Or update specific submodule
cd plugins/rust-skills && git pull origin main
```

## Adding New Sub-Plugins

```bash
cd cowork-skills

# Add new plugin as submodule
git submodule add https://github.com/user/makepad-skills.git plugins/makepad-skills

# Sync skills symlinks
./sync-skills.sh
```

## Using Individual Plugins

Each plugin can also be used standalone:

```bash
claude --plugin-dir /path/to/cowork-skills/plugins/rust-skills
claude --plugin-dir /path/to/cowork-skills/plugins/makepad-skills
```

## License

MIT License

## Links

- **rust-skills**: https://github.com/ZhangHanDong/rust-skills
- **Issues**: https://github.com/ZhangHanDong/cowork-skills/issues
