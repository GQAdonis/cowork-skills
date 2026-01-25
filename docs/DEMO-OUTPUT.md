# CoWork CLI - 命令输出示例

> 用于介绍视频的完整命令输出参考

## 目录

1. [Quick Start 快速开始](#1-quick-start-快速开始)
2. [Install 安装管理](#2-install-安装管理)
3. [Configuration 项目配置](#3-configuration-项目配置)
4. [Discovery 发现 Skills](#4-discovery-发现-skills)
5. [Plugin Management 插件管理](#5-plugin-management-插件管理)
6. [Security 安全测试](#6-security-安全测试)
7. [Built-in Skills 内置技能](#7-built-in-skills-内置技能)

---

## 1. Quick Start 快速开始

### `co init --list` - 列出内置 Skills

```bash
❯ co init --list
Available Built-in Skills

  ● memory-skills
    CoALA cognitive architecture memory system
  ● cowork-guide
    CoWork CLI usage guide and documentation
  ● cowork-router
    Unified router for installed plugins/skills
  ● code-review
    Code review assistant with best practices
  ● github-generate
    Generate skills from GitHub repositories
  ● github-search
    Search GitHub for skill repositories

→ Use 'cowork init' to install all (global)
→ Use 'cowork init --local' to install to project
→ Use 'cowork init -s <name>' to install specific skill
→ Use 'cowork init -r <name>' to remove a skill
```

### `co init` - 安装所有内置 Skills (全局)

```bash
❯ co init
Initializing CoWork

→ Installing built-in skills to ~/.claude/skills/

  ✓ memory-skills
  ✓ cowork-guide
  ✓ cowork-router
  ✓ code-review
  ✓ github-generate
  ✓ github-search

Summary:
  ✓ Installed: 6

✓ Skills installed to ~/.claude/skills/
ℹ These skills are now available globally in Claude Code
```

### `co init --local` - 安装到项目

```bash
❯ co init --local
Initializing CoWork

→ Installing built-in skills to .claude/skills/

  ✓ memory-skills
  ✓ cowork-guide
  ✓ cowork-router
  ✓ code-review
  ✓ github-generate
  ✓ github-search

Summary:
  ✓ Installed: 6

✓ Skills installed to .claude/skills/
ℹ These skills are available in the current project
```

### `co init -s memory-skills` - 安装指定 Skill

```bash
❯ co init -s memory-skills
Initializing CoWork

→ Installing built-in skills to ~/.claude/skills/

  ✓ memory-skills

Summary:
  ✓ Installed: 1

✓ Skills installed to ~/.claude/skills/
ℹ These skills are now available globally in Claude Code
```

### `co init -r memory-filesystem` - 删除 Skill

```bash
❯ co init -r memory-filesystem
Removing Skills

→ Removing from ~/.claude/skills/

  ✓ memory-filesystem

Summary:
  ✓ Removed: 1
```

---

## 2. Install 安装管理

### `co install user/repo` - 从 GitHub 安装

```bash
❯ co install ZhangHanDong/rust-skills
Installing from GitHub

→ Cloning ZhangHanDong/rust-skills...
  ✓ Cloned to ~/.cowork/repos/rust-skills

→ Detecting skills...
  Found 25 skills in skills/ directory

→ Installing to ~/.claude/skills/
  ✓ m01-ownership
  ✓ m02-resource
  ✓ m03-mutability
  ...
  ✓ rust-router

Summary:
  ✓ Installed: 25 skills
  → Location: ~/.claude/skills/

ℹ Skills are now available in Claude Code
```

### `co install user/repo --plugin` - 作为插件安装

```bash
❯ co install ZhangHanDong/rust-skills --plugin
Installing as plugin

→ Cloning ZhangHanDong/rust-skills...
  ✓ Cloned to ~/.cowork/repos/rust-skills

→ Creating plugin symlink...
  ✓ ~/.claude/rust-skills -> ~/.cowork/repos/rust-skills

Summary:
  ✓ Installed plugin: rust-skills
  → Contains: 25 skills

ℹ Plugin is now available in Claude Code
```

### `co install user/repo -l` - 安装到项目本地

```bash
❯ co install ZhangHanDong/rust-skills -l
Installing to project local

→ Cloning ZhangHanDong/rust-skills...
  ✓ Cloned to ~/.cowork/repos/rust-skills

→ Installing to .claude/skills/
  ✓ m01-ownership
  ✓ m02-resource
  ...

Summary:
  ✓ Installed: 25 skills
  → Location: .claude/skills/

ℹ Skills are available in the current project only
```

### `co install --list` - 列出已安装仓库

```bash
❯ co install --list
Installed Repositories

  ~/.cowork/repos/
  ├── rust-skills (ZhangHanDong/rust-skills)
  │   └── 25 skills, installed to ~/.claude/skills/
  ├── dora-skills (dora-rs/dora-skills)
  │   └── 17 skills, installed to ~/.claude/dora-skills/ (plugin)
  └── makepad-skills (user/makepad-skills)
      └── 11 skills, installed to .claude/skills/ (local)

Total: 3 repositories, 53 skills
```

### `co install --uninstall repo` - 卸载仓库

```bash
❯ co install --uninstall rust-skills
Uninstalling rust-skills

→ Removing skills from ~/.claude/skills/
  ✓ Removed 25 skills

→ Removing repository
  ✓ Removed ~/.cowork/repos/rust-skills

Summary:
  ✓ Uninstalled: rust-skills
```

---

## 3. Configuration 项目配置

### `co config init` - 创建 Skills.toml

```bash
❯ co config init
Detected installed plugins/skills:

  Global ~/.claude/ (plugins):
    [1] rust-skills v0.1.0
    [2] dora-skills v0.2.0 (symlink)

  Global ~/.claude/skills/ (skills):
    [3] memory-skills
    [4] cowork-guide

? Include in Skills.toml? [Y/n/select 1,2,3]: Y

✓ Created .cowork/Skills.toml
  4 packages configured

Next steps:
  1. Edit .cowork/Skills.toml to configure your skills
  2. Run 'cowork config show' to verify
  3. Run 'cowork config install' to install dependencies
```

### `co config show` - 显示当前配置

```bash
❯ co config show
Skills Configuration

Project: my-project
Description: My awesome project

Global Skills (~/.claude/skills/)
  Enabled: memory-skills, cowork-guide
  Disabled: (none)

Skill Groups
  Enabled: rust-core
  Disabled: rust-domains

Dependencies
  rust-skills = "ZhangHanDong/rust-skills" (global)
  dora-skills = "/path/to/dora-skills" (local, plugin)

Dev Links
  my-test -> "/path/to/test-skill" (local)

Triggers
  Priority: dora-router > rust-router
  Overrides:
    "async" -> rust-router
```

### `co config add` - 添加依赖

```bash
❯ co config add rust ZhangHanDong/rust-skills
✓ Added dependency: rust = "ZhangHanDong/rust-skills"

→ Run 'cowork config install' to install
```

```bash
❯ co config add dora-dev /path/to/dora-skills --dev --plugin
✓ Added dev link: dora-dev -> "/path/to/dora-skills" (plugin, local)

→ Run 'cowork config install' to install
```

### `co config install` - 安装依赖

```bash
❯ co config install
Installing dependencies from Skills.toml

→ rust-skills (global)...
  ✓ Installed

→ dora-skills (local, plugin)...
  ✓ Installed

Creating dev links

→ dora-dev -> /path/to/dora-skills (plugin, local)...
  ✓ Linked

✓ Updated .cowork/Skills.lock
  2 installed, 1 linked
```

### `co config router --analyze` - AI 分析生成路由

```bash
❯ co config router --analyze
Generating AI-enhanced cowork-router...

  → Running AI analysis...
  ✓ Cached analysis for future use

✓ Generated .claude/skills/cowork-router/SKILL.md
  3 plugins with 47 total skills
```

---

## 4. Discovery 发现 Skills

### `co list` - 列出所有 Skills

```bash
❯ co list
Available Skills

Global (~/.claude/skills/):
  ● memory-skills        Memory management with CoALA architecture
  ● cowork-guide         CoWork CLI complete guide
  ● cowork-router        Unified router for plugins
  ● code-review          Code review assistant
  ● m01-ownership        Rust ownership and borrowing
  ● m02-resource         Smart pointers and RAII
  ...

Project (.claude/skills/):
  ● my-custom-skill      Custom project skill

Plugins:
  ● rust-skills (25 skills)
  ● dora-skills (17 skills)

Total: 48 skills
```

### `co search tokio` - 搜索 GitHub

```bash
❯ co search tokio --limit 5
Searching GitHub for 'tokio'...

Results (5):

  tokio-rs/tokio ⭐ 25.3k
    A runtime for writing reliable, asynchronous applications
    Topics: async, rust, runtime

  tokio-rs/axum ⭐ 18.2k
    Ergonomic and modular web framework
    Topics: web, http, async

  tokio-rs/tracing ⭐ 5.1k
    Application level tracing for Rust
    Topics: logging, tracing

→ Use 'cowork install tokio-rs/tokio' to install
```

### `co generate user/repo` - 从源码生成

```bash
❯ co generate tokio-rs/tokio --lang rust
Generating skills from tokio-rs/tokio

→ Cloning repository...
  ✓ Cloned to ~/.cowork/repos/tokio

→ Parsing Rust source code...
  Scanning 156 .rs files
  Found 1,247 public items

→ Generating llms.txt...
  ✓ Created llms.txt (245 KB)

→ Generating skills...
  ✓ tokio-runtime (Runtime management)
  ✓ tokio-sync (Synchronization primitives)
  ✓ tokio-io (Async I/O)
  ✓ tokio-net (Networking)
  ✓ tokio-time (Time and delays)

Summary:
  ✓ Generated: 5 skills
  → Output: ~/.cowork/generated/tokio/
```

---

## 5. Plugin Management 插件管理

### `co plugins list` - 列出插件

```bash
❯ co plugins list
Installed Plugins

  rust-skills@rust-skills
    Version: 0.1.0
    Scope: user (global)
    Path: ~/.claude/plugins/cache/rust-skills-abc123/
    Status: ✓ enabled

  dora-skills@dora-skills
    Version: 0.2.0
    Scope: local (project)
    Path: .claude/dora-skills/
    Status: ✓ enabled

Total: 2 plugins
```

### `co plugins status` - 插件状态

```bash
❯ co plugins status
Plugin System Status

Configuration Files:
  ✓ ~/.claude/plugins/installed_plugins.json
  ✓ ~/.claude/settings.json

Installed Plugins: 2
  ✓ rust-skills (enabled)
  ✓ dora-skills (enabled)

Marketplaces: 1
  ● rust-skills (local)

Cache Directory: ~/.claude/plugins/cache/
  Size: 12.5 MB
```

### `co plugins enable/disable` - 启用/禁用

```bash
❯ co plugins disable rust-skills
✓ Disabled plugin: rust-skills

❯ co plugins enable rust-skills
✓ Enabled plugin: rust-skills
```

---

## 6. Security 安全测试

### `co audit` - 安全审计

```bash
❯ co audit
Security Audit

Scanning skills...
  → Global: ~/.claude/skills/ (8 skills)
  → Project: .claude/skills/ (2 skills)
  → Plugins: 2 plugins (42 skills)

Results:

  ✓ memory-skills                    SAFE
  ✓ cowork-guide                     SAFE
  ⚠ untrusted-skill                  MEDIUM
    └─ Line 45: Contains 'eval()' pattern
  ✓ rust-skills (plugin)             SAFE

Summary:
  ✓ Safe: 51
  ⚠ Low: 0
  ⚠ Medium: 1
  ✗ High: 0
  ✗ Critical: 0
```

### `co verify` - 校验 Checksums

```bash
❯ co verify
Verifying Skills

  ✓ rust-skills          SHA256 matches
  ✓ dora-skills          SHA256 matches
  ✗ modified-skill       MISMATCH
    Expected: abc123...
    Actual:   def456...

Summary:
  ✓ Verified: 2
  ✗ Failed: 1

⚠ Some skills have been modified since installation
→ Use 'cowork verify --update' to update checksums
```

### `co test --check-conflicts` - 检查冲突

```bash
❯ co test --check-conflicts
Trigger Conflict Analysis

Checking 52 skills for trigger conflicts...

Conflicts Found:

  "async" triggers multiple skills:
    → rust-skills:m07-concurrency (priority: 1)
    → tokio-skills:tokio-runtime (priority: 2)

  "error handling" triggers multiple skills:
    → rust-skills:m06-error-handling (priority: 1)
    → general:error-guide (priority: 2)

Recommendations:
  1. Set priority: cowork config priority rust-router tokio-router
  2. Override specific: cowork config override "async" m07-concurrency

Total: 2 conflicts in 52 skills
```

### `co doctor` - 诊断问题

```bash
❯ co doctor
CoWork Health Check

Configuration:
  ✓ Skills.toml exists
  ✓ Skills.lock exists
  ✓ Lock file is in sync

Directories:
  ✓ ~/.claude/skills/ (8 skills)
  ✓ ~/.cowork/repos/ (3 repos)
  ✓ .claude/skills/ (2 skills)

Plugins:
  ✓ rust-skills (enabled)
  ✓ dora-skills (enabled)

Environment:
  ✓ GITHUB_TOKEN is set
  ✓ claude command available

All checks passed!
```

---

## 7. Built-in Skills 内置技能

### memory-skills - 记忆系统

**触发词**: remember, recall, 记住, 回忆, save context, 保存上下文

```
用户: 记住：Rust 中使用 thiserror 来定义错误类型

Claude: ✓ 已保存到语义记忆
        位置: ~/.claude/memory/semantic/learnings/rust-tips.md
        类型: Semantic (知识/技巧)
        标签: rust, error, thiserror
```

```
用户: 保存上下文 cowork-router

Claude: ✓ 已保存上下文到 ~/.claude/memory/working/cowork-router-context.md

        包含:
        - 当前目标: 实现 AI 分析路由
        - 关键决策: 使用 claude -p 调用
        - 下一步: 测试缓存功能
```

```
用户: 回忆一下 rust 错误处理

Claude: 找到 2 条相关记忆:

        📁 [Semantic] ~/.claude/memory/semantic/learnings/rust-tips.md
           → "使用 thiserror 来定义错误类型..."

        📁 [Episodic] ~/.claude/memory/episodic/solutions/2026-01-20-error-fix.md
           → "修复 anyhow 转换问题..."
```

### cowork-guide - CLI 指南

**触发词**: cowork, Skills.toml, skill management, 技能管理

```
用户: 怎么用 cowork 安装 skills?

Claude: [自动加载 cowork-guide skill]

        CoWork 安装命令:

        # 从 GitHub 安装
        co install user/repo

        # 安装特定 skills
        co install user/repo -s skill1 -s skill2

        # 作为插件安装
        co install user/repo --plugin

        完整指南请使用 'co --help'
```

### code-review - 代码审查

**触发词**: review PR, code review, PR review, 代码审查

```
用户: /code-review #123

Claude: [自动加载 code-review skill]

        PR #123 代码审查

        ## 概述
        - 文件变更: 5 files (+120, -45)
        - 作者: @developer
        - 分支: feature/new-api -> main

        ## 发现

        ✓ 优点:
        - 良好的错误处理
        - 清晰的函数命名

        ⚠ 建议:
        - src/api.rs:45 - 考虑使用 `?` 替代 unwrap
        - src/db.rs:78 - 缺少事务处理

        ## 总结
        整体质量良好，有 2 个小建议需要处理。
```

### cowork-router - 统一路由

**触发词**: (自动检测已安装插件的关键词)

```
用户: Rust 中 E0382 错误怎么解决?

Claude: [cowork-router 检测到 rust 关键词]
        [路由到 rust-skills:m01-ownership]

        E0382: 使用已移动的值

        错误原因:
        尝试使用一个所有权已经被转移的值...

        解决方案:
        1. 使用 .clone()
        2. 使用引用 &
        3. 实现 Copy trait
```

---

## 完整演示流程

```bash
# 1. 安装 CoWork
cd cowork-skills/cli
cargo install --path .

# 2. 初始化内置 skills
co init

# 3. 安装外部 skills
co install ZhangHanDong/rust-skills

# 4. 创建项目配置
cd my-project
co config init

# 5. 添加依赖
co config add dora dora-rs/dora-skills --plugin

# 6. 安装配置的依赖
co config install

# 7. 生成智能路由
co config router --analyze

# 8. 检查配置
co status
co doctor

# 9. 安全审计
co audit

# 10. 查看所有可用 skills
co list
```
