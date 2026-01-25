# Memory Filesystem

[![CoALA](https://img.shields.io/badge/Based%20on-CoALA-blue)](https://arxiv.org/abs/2309.02427)

> **Memory = Filesystem** - A cognitive architecture for language agents

[中文](./README-zh.md) | [日本語](./README-ja.md)

## Overview

Memory Filesystem is a skill for Claude Code that implements a complete memory system based on the [CoALA framework](https://arxiv.org/abs/2309.02427) (Cognitive Architectures for Language Agents) from Princeton University.

## Features

- **Four Memory Types**: Working, Procedural, Semantic, Episodic
- **Dual Scope**: Global (`~/.claude/memory/`) and Project (`.claude/memory/`)
- **Smart Detection**: Auto-categorizes memories by content type
- **Reflection**: Converts episodic experiences into semantic knowledge
- **Forgetting**: Removes outdated or irrelevant memories
- **Search Index**: JSON-based index for fast retrieval

## Installation

```bash
# Install cowork CLI
cargo install cowork

# Initialize built-in skills
cowork init
```

## Memory Architecture

```
                    ┌─────────────────┐
                    │  Working Memory │  ← Short-term
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        ↓                    ↓                    ↓
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│   Episodic    │   │   Semantic    │   │  Procedural   │
│  (Experiences)│   │  (Knowledge)  │   │  (How-to)     │
└───────────────┘   └───────────────┘   └───────────────┘
```

## Commands

| Command | Description |
|---------|-------------|
| `/remember <content>` | Save to appropriate memory location |
| `/recall <topic>` | Search and retrieve memories |
| `/forget <topic>` | Delete specific memories |
| `/reflect` | Generate insights from experiences |
| `/summarize-session` | Save current session summary |

## Directory Structure

### Global Memory (`~/.claude/memory/`)

```
~/.claude/memory/
├── working/                    # Short-term session context
├── procedural/                 # Preferences, workflows, prompts
├── semantic/                   # Learnings, reflections
├── episodic/                   # Mistakes, solutions, trajectories
└── index.json                  # Search index
```

### Project Memory (`.claude/memory/`)

```
.claude/memory/
├── semantic/                   # Architecture, conventions, decisions
├── episodic/                   # Session summaries
└── index.json                  # Project search index
```

## Usage Examples

### Remember something
```
User: "Remember: Use thiserror for defining error types in Rust"
Claude: ✓ Saved to semantic memory (rust-tips.md)
```

### Recall information
```
User: "What do you know about error handling?"
Claude: Found 2 relevant memories:
  📁 [Semantic] rust-tips.md → "Use thiserror for..."
  📁 [Episodic] solutions/2026-01-20-error-fix.md → "Fixed by..."
```

### Reflect on experiences
```
User: "/reflect"
Claude: ✓ Generated 1 reflection from 3 recent episodes
        Saved to: semantic/reflections/2026-01-25-async-patterns.md
```

## Memory Types Explained

| Type | Purpose | Examples |
|------|---------|----------|
| **Working** | Current session state | Active goals, intermediate results |
| **Procedural** | How to do things | User preferences, workflows |
| **Semantic** | Facts and knowledge | Tips, learnings, reflections |
| **Episodic** | Past experiences | Mistakes made, problems solved |

## License

MIT
