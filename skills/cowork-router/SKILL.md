---
name: cowork-router
description: "Unified router for all Rust domain skills - routes to rust-skills, makepad-skills, dora-skills based on context"
---

# CoWork Router

> Master router for integrated Rust development across multiple domains

## Purpose

This router detects the user's domain context and routes to the appropriate skill set:
- **rust-skills**: Core Rust language mechanics (ownership, concurrency, error handling)
- **makepad-skills**: Makepad UI framework development
- **dora-skills**: Dora-rs robotics framework development

## Domain Detection

| Domain | Keywords | Route To |
|--------|----------|----------|
| **Makepad UI** | makepad, widget, view, live_design, Draw2d, WidgetRef | `makepad-router` |
| **Dora Robotics** | dora, node, operator, dataflow, robot, sensor | `dora-router` |
| **Rust Core** | E0xxx, ownership, borrow, lifetime, async, trait | `rust-router` |

## Cross-Domain Questions

When a question involves multiple domains, load skills from both:

### Example: "Makepad 中如何处理 E0382 错误"

**Analysis:**
- Primary domain: Makepad (UI context)
- Sub-question: E0382 (Rust ownership error)

**Action:**
1. Load `makepad-router` for UI component lifecycle context
2. Load `m01-ownership` for ownership mechanics
3. Combine: Explain ownership in the context of Makepad widget lifecycle

### Example: "Dora node 中数据所有权如何设计"

**Analysis:**
- Primary domain: Dora (robotics context)
- Sub-question: Ownership design

**Action:**
1. Load `dora-router` for dataflow architecture context
2. Load `m01-ownership` + `m09-domain` for ownership and domain modeling
3. Combine: Design ownership for real-time robotics constraints

## Routing Rules

### Priority Order

```
1. Check for domain-specific keywords (Makepad, Dora)
   ├── If found with Rust error codes → Cross-domain handling
   └── If found alone → Domain-specific router

2. Check for Rust core keywords
   └── Route to rust-router

3. No match
   └── Let Claude handle with general knowledge
```

### Skill Inheritance

All domain skills automatically inherit from rust-skills:

```
rust-skills (Layer 1-2)
├── m01-m07: Language Mechanics
├── m09-m15: Design Patterns
└── domains/*: Domain Constraints
        │
        ├──────────────────┐
        ▼                  ▼
makepad-skills         dora-skills
├── UI constraints     ├── Robotics constraints
├── Widget patterns    ├── Dataflow patterns
└── ← inherits rust    └── ← inherits rust
```

## Usage

### Direct Invocation

```
/cowork-router
```

### Auto-Triggered

The hook script automatically detects domain keywords and invokes this router.

## Related Skills

| Skill | Domain | Description |
|-------|--------|-------------|
| `rust-router` | Rust Core | Routes to m01-m15 based on Rust keywords |
| `makepad-router` | UI | Routes to Makepad-specific skills |
| `dora-router` | Robotics | Routes to Dora-specific skills |
