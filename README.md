# CoWork Skills

[中文](#中文) | [日本語](#日本語)

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
│   └── cowork-router/     # Master router
├── plugins/
│   ├── rust-skills/       # Git submodule - Core Rust
│   ├── makepad-skills/    # Git submodule - UI
│   └── dora-skills/       # Git submodule - Robotics
└── .claude/hooks/         # Unified hooks
```

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

Copy the example settings:

```bash
cp /path/to/cowork-skills/plugins/rust-skills/.claude/settings.example.json .claude/settings.local.json
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

# Or update specific submodule
cd plugins/rust-skills && git pull origin main
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

---

<a name="中文"></a>
# CoWork Skills (中文)

> 多领域集成的 Rust 开发助手

## 简介

**CoWork Skills** 是一个父插件，集成多个 Rust 领域技能：

- **rust-skills** - Rust 核心语言知识
- **makepad-skills** - Makepad UI 框架开发
- **dora-skills** - Dora-rs 机器人框架开发

## 安装

```bash
# 克隆（包含子模块）
git clone --recurse-submodules https://github.com/ZhangHanDong/cowork-skills.git

# 启动
claude --plugin-dir /path/to/cowork-skills
```

## 工作原理

```
用户问题 → cowork-router → 检测领域 → 路由到对应技能 → 领域感知的回答
```

## 跨领域问题

当问题涉及多个领域时，CoWork 会同时加载相关技能：

```
用户: "Makepad widget 中如何解决 E0382"

路由:
├── 主领域: Makepad (UI 上下文)
├── 子问题: E0382 (Rust 所有权)
└── 操作: 加载两个技能，结合知识回答
```

---

<a name="日本語"></a>
# CoWork Skills (日本語)

> マルチドメイン統合 Rust 開発アシスタント

## 概要

**CoWork Skills** は複数の Rust ドメインスキルを統合する親プラグインです：

- **rust-skills** - Rust コア言語知識
- **makepad-skills** - Makepad UI フレームワーク開発
- **dora-skills** - Dora-rs ロボティクスフレームワーク開発

## インストール

```bash
# クローン（サブモジュール含む）
git clone --recurse-submodules https://github.com/ZhangHanDong/cowork-skills.git

# 起動
claude --plugin-dir /path/to/cowork-skills
```

## 動作原理

```
ユーザー質問 → cowork-router → ドメイン検出 → 対応スキルへルーティング → ドメイン認識回答
```

## クロスドメイン質問

複数のドメインにまたがる質問の場合、関連するスキルを同時にロード：

```
ユーザー: "Makepad widget で E0382 を解決するには"

ルーティング:
├── メインドメイン: Makepad (UI コンテキスト)
├── サブ質問: E0382 (Rust 所有権)
└── アクション: 両方のスキルをロードし、知識を組み合わせて回答
```
