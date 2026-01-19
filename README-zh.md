# CoWork Skills

[English](./README.md) | [日本語](./README-ja.md)

> 多领域集成的 Rust 开发助手

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-Plugin-blue)](https://github.com/anthropics/claude-code)

## 什么是 CoWork Skills？

**CoWork Skills** 是一个父插件，将多个 Rust 领域技能集成到统一的开发体验中：

- **rust-skills** - Rust 核心语言知识（所有权、并发、错误处理）
- **makepad-skills** - Makepad UI 框架开发
- **dora-skills** - Dora-rs 机器人框架开发

## 架构

```
cowork-skills/
├── skills/
│   ├── cowork-router/     # 主路由器 (原生)
│   ├── rust-router/       # → 符号链接到 plugins/rust-skills/skills/
│   ├── m01-ownership/     # → 符号链接到 plugins/rust-skills/skills/
│   └── ...                # 所有子插件 skills 通过符号链接
├── plugins/
│   ├── rust-skills/       # Git submodule - Rust 核心
│   ├── makepad-skills/    # Git submodule - UI
│   └── dora-skills/       # Git submodule - 机器人
├── .claude/hooks/         # 统一 hooks
└── sync-skills.sh         # 同步符号链接脚本
```

> **注意**: Claude Code 只加载插件根目录的 `skills/`。我们使用符号链接来包含子插件的 skills。

## 安装

### 克隆（包含子模块）

```bash
git clone --recurse-submodules https://github.com/ZhangHanDong/cowork-skills.git
```

### 启动 Claude Code

```bash
claude --plugin-dir /path/to/cowork-skills
```

### 权限配置

复制示例配置到你的项目：

```bash
cp /path/to/cowork-skills/.claude/settings.example.json .claude/settings.local.json
```

## 工作原理

```
用户问题
     │
     ▼
┌─────────────────────────────────┐
│       cowork-router-hook        │
│       从关键词检测领域            │
└─────────────────────────────────┘
     │
     ├─────────────┬─────────────┐
     ▼             ▼             ▼
┌─────────┐  ┌─────────┐  ┌─────────┐
│ Makepad │  │  Dora   │  │  Rust   │
│ 路由器   │  │ 路由器   │  │ 路由器   │
└─────────┘  └─────────┘  └─────────┘
     │             │             │
     └─────────────┴─────────────┘
                   │
                   ▼
           领域感知的回答
```

## 领域技能

| 领域 | 插件 | 描述 |
|------|------|------|
| **Rust 核心** | rust-skills | 所有权、并发、错误处理、元认知框架 |
| **UI 开发** | makepad-skills | Makepad widgets、views、live design |
| **机器人** | dora-skills | Dora nodes、operators、dataflow |

## 跨领域问题

CoWork Skills 处理涉及多个领域的问题：

```
用户: "Makepad widget 中如何处理 E0382？"

CoWork 路由:
├── 主领域: Makepad (UI 上下文)
├── 子问题: E0382 (Rust 所有权)
└── 操作: 加载两个技能，结合知识回答
```

## 更新子模块

```bash
cd cowork-skills

# 更新所有子模块
git submodule update --remote

# 更新后重新同步符号链接
./sync-skills.sh

# 或更新特定子模块
cd plugins/rust-skills && git pull origin main
```

## 添加新子插件

```bash
cd cowork-skills

# 添加新插件为 submodule
git submodule add https://github.com/user/makepad-skills.git plugins/makepad-skills

# 同步符号链接
./sync-skills.sh
```

## 单独使用插件

每个插件也可以独立使用：

```bash
claude --plugin-dir /path/to/cowork-skills/plugins/rust-skills
claude --plugin-dir /path/to/cowork-skills/plugins/makepad-skills
```

## 许可证

MIT 许可证

## 链接

- **rust-skills**: https://github.com/ZhangHanDong/rust-skills
- **Issues**: https://github.com/ZhangHanDong/cowork-skills/issues
