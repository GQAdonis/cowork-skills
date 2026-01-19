# CoWork Skills - Claude Instructions

## Overview

CoWork Skills is a parent plugin that integrates multiple Rust domain skills:
- **rust-skills**: Core Rust language (ownership, concurrency, error handling)
- **makepad-skills**: Makepad UI framework
- **dora-skills**: Dora-rs robotics framework

## CRITICAL: Routing Priority

For ANY question in a Rust project, follow this routing order:

```
1. cowork-router    → Detect domain context
2. Domain router    → rust-router / makepad-router / dora-router
3. Specific skill   → m01-ownership / makepad-widget / dora-node
```

## Domain Detection

| Keywords | Domain | Router |
|----------|--------|--------|
| makepad, widget, view, live_design | UI | makepad-router |
| dora, node, operator, dataflow | Robotics | dora-router |
| E0xxx, ownership, async, trait | Rust Core | rust-router |

## Skill Inheritance

All domain skills inherit from rust-skills:

```
rust-skills (Base)
├── Layer 1: m01-m07 (Language Mechanics)
├── Layer 2: m09-m15 (Design Patterns)
└── Layer 3: domains/* (Domain Constraints)
        │
        ├──────────────────┐
        ▼                  ▼
makepad-skills         dora-skills
(UI Domain)            (Robotics Domain)
```

## Cross-Domain Questions

When a question involves multiple domains:

1. **Identify primary domain** - What is the main context?
2. **Load primary router** - Get domain-specific constraints
3. **Load Rust skill** - Get language mechanics
4. **Combine knowledge** - Answer with full context

### Example

```
User: "Makepad widget 中如何解决 E0382"

Analysis:
- Primary: Makepad (widget context)
- Secondary: E0382 (ownership error)

Action:
1. Load makepad-router → Widget lifecycle
2. Load m01-ownership → Ownership mechanics
3. Combine → Ownership design for UI components
```

## Available Sub-Plugins

### rust-skills (plugins/rust-skills/)
- `rust-router` - Core Rust question router
- `m01-m07` - Language mechanics (ownership, concurrency, etc.)
- `m09-m15` - Design patterns (domain modeling, performance, etc.)
- `domains/*` - Domain constraints (fintech, web, CLI, etc.)
- `rust-learner` - Latest Rust/crate version info

### makepad-skills (plugins/makepad-skills/)
- `makepad-router` - Makepad question router
- Widget, View, LiveDesign patterns
- UI performance optimization

### dora-skills (plugins/dora-skills/)
- `dora-router` - Dora question router
- Node, Operator, Dataflow patterns
- Real-time robotics constraints

## Default Project Settings

When creating Rust projects, use:

```toml
[package]
edition = "2024"
rust-version = "1.85"

[lints.clippy]
all = "warn"
pedantic = "warn"
```
