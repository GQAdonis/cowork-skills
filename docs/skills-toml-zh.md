# Skills.toml 配置指南

`Skills.toml` 是项目级别的 Claude Code 技能配置文件，类似于 Rust 项目的 Cargo.toml。文件存储在项目根目录的 `.cowork/Skills.toml`。

## 工作流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     cowork config 工作流程                       │
└─────────────────────────────────────────────────────────────────┘

1. 安装 cowork CLI
   $ cargo install --git https://github.com/anthropics/cowork-skills cli

2. 在项目中初始化配置
   $ cd your-project
   $ cowork config init

   创建: your-project/.cowork/Skills.toml

3. 配置技能（编辑 .cowork/Skills.toml 或使用 CLI）
   $ cowork config enable rust-core makepad
   $ cowork config add rust-skills ZhangHanDong/rust-skills

4. 安装技能依赖
   $ cowork config install

5. 应用配置（可选 - 生成 SKILLS.md）
   $ cowork config apply

初始化后的目录结构：
   your-project/
   ├── .cowork/
   │   ├── Skills.toml      # 技能配置
   │   └── Skills.lock      # 已安装技能锁定文件（自动生成）
   ├── skills/              # 本地项目技能（可选）
   └── ...
```

## 快速开始

```bash
# 在项目中初始化 .cowork/Skills.toml
cowork config init

# 查看当前配置
cowork config show

# 添加技能依赖
cowork config add rust-skills ZhangHanDong/rust-skills

# 安装所有依赖
cowork config install
```

## 文件位置

配置文件存储位置：
- **项目配置**: `.cowork/Skills.toml`（项目根目录）
- **锁定文件**: `.cowork/Skills.lock`（自动生成）
- **全局技能**: `~/.claude/skills/`（跨项目共享）
- **本地技能**: `./skills/`（项目特定）

## Skills.lock

`Skills.lock` 在运行 `cowork config install` 时自动生成。它记录已安装技能/插件的确切版本和状态。

```toml
# 此文件由 cowork 自动生成，请勿手动编辑。

version = 1

[[package]]
name = "rust-skills"
version = "1.2.0"
source = "github"
scope = "project"
type = "plugin"
install_path = "/path/to/project/.claude/rust-skills"
source_path = "ZhangHanDong/rust-skills"
git_sha = "abc123..."
installed_at = "2024-01-25T10:30:00Z"
updated_at = "2024-01-25T10:30:00Z"
enabled = true
skills = ["m01-ownership", "m02-resource", "rust-router"]

[[package]]
name = "dora-dev"
version = "20240125.103000"
source = "dev"
scope = "project"
type = "symlink"
install_path = "/path/to/project/.claude/skills/dora-dev"
source_path = "/local/path/to/dora-skills"
installed_at = "2024-01-25T10:30:00Z"
updated_at = "2024-01-25T10:30:00Z"
enabled = true
```

**锁定文件字段说明：**

| 字段 | 描述 |
|------|------|
| `name` | 包名称 |
| `version` | 来自 plugin.json/package.json 的版本，或安装时间戳 |
| `source` | `github`、`local` 或 `dev` |
| `scope` | `global`（全局）或 `project`（项目） |
| `type` | `plugin`、`skills` 或 `symlink` |
| `install_path` | 安装位置 |
| `source_path` | 原始仓库或本地路径 |
| `git_sha` | Git 提交 SHA（GitHub 来源） |
| `installed_at` | 安装时间戳 |
| `updated_at` | 最后更新时间戳 |
| `enabled` | 是否启用 |
| `skills` | 已安装的技能列表 |

## 文件结构

```toml
# Skills.toml - 项目技能配置

[project]
name = "my-project"
description = "项目描述"

[skills.global]
enabled = ["memory-filesystem", "best-skill-creator"]
disabled = []

[skills.install]
rust-skills = "ZhangHanDong/rust-skills"

[skills.groups]
enabled = ["rust-core"]
disabled = []

[triggers]
priority = ["dora-router", "rust-router"]

[triggers.overrides]
"async" = "rust-router"
```

## 配置节说明

### [project]

项目元数据。

| 字段 | 类型 | 描述 |
|------|------|------|
| `name` | string | 项目名称（从目录名自动检测） |
| `description` | string | 可选的项目描述 |

### [skills.global]

配置全局技能（来自 `~/.claude/skills/`）。

| 字段 | 类型 | 描述 |
|------|------|------|
| `enabled` | array | 白名单 - 只加载这些技能 |
| `disabled` | array | 黑名单 - 排除这些技能 |

**规则：**
- 如果 `enabled` 为空，所有全局技能可用
- 如果设置了 `enabled`，只加载列出的技能
- `disabled` 始终排除技能，即使在 `enabled` 中

### [skills.install]

声明来自 GitHub 或本地路径的技能依赖。

**简单形式：**
```toml
[skills.install]
rust-skills = "ZhangHanDong/rust-skills"
```

**详细形式：**
```toml
[skills.install]
# 从 GitHub 安装指定技能
tokio = { repo = "user/tokio-skills", skills = ["tokio-runtime", "tokio-sync"] }

# 从本地路径
local = { path = "../my-local-skills" }

# 固定到特定版本
pinned = { repo = "user/repo", ref = "v1.0.0" }

# 指定目标代理
multi = { repo = "user/repo", agents = ["claude-code", "cursor"] }
```

**插件形式（保留完整仓库结构）：**
```toml
[skills.install]
# 作为插件安装 - 保留整个仓库结构
makepad = { repo = "user/makepad-skills", plugin = true }

# 带版本固定的插件
dora = { repo = "user/dora-skills", plugin = true, ref = "v2.0" }
```

**安装位置（全局 vs 本地）：**
```toml
[skills.install]
# 安装到全局 ~/.claude/skills/（默认）
rust-skills = "ZhangHanDong/rust-skills"

# 安装到项目 .claude/skills/
my-project = { repo = "user/skills", local = true }
my-plugin = { repo = "user/plugin", plugin = true, local = true }
```

**禁用的依赖（已安装但未启用）：**
```toml
[skills.install]
old-lib = { repo = "user/old", enabled = false }
```

| 字段 | 类型 | 描述 |
|------|------|------|
| `repo` | string | GitHub 仓库（user/repo） |
| `path` | string | 本地文件系统路径 |
| `skills` | array | 要安装的特定技能（默认：全部） |
| `ref` | string | Git 引用（分支、标签、提交） |
| `agents` | array | 目标 AI 代理 |
| `plugin` | bool | 作为插件安装（保留完整仓库结构） |
| `local` | bool | 安装到项目而非全局（默认：false） |
| `enabled` | bool | 启用/禁用此依赖（默认：true） |

### [skills.dev]

开发链接（符号链接）用于在开发期间测试本地技能。

```toml
[skills.dev]
# 简单形式：链接到项目 .claude/skills/
my-skill = "/path/to/my-skill-project"

# 详细形式：
dora-dev = { path = "/path/to/dora-skills" }
disabled-dev = { path = "/path/to/skills", enabled = false }

# 插件形式：链接到 .claude/<name>/ 而不是 .claude/skills/
dora-plugin = { path = "/path/to/dora-skills", plugin = true }
```

| 字段 | 类型 | 描述 |
|------|------|------|
| `path` | string | 要链接的本地路径 |
| `local` | bool | 链接到项目（默认：true） |
| `enabled` | bool | 启用/禁用此链接（默认：true） |
| `plugin` | bool | 作为插件链接到 `.claude/<name>/`（默认：false） |

**链接位置：**
- `plugin = false`（默认）：`.claude/skills/<name>`（技能）
- `plugin = true`：`.claude/<name>/`（插件）

### [skills.groups]

启用或禁用预定义的技能组。

```toml
[skills.groups]
enabled = ["rust-core", "makepad"]
disabled = ["rust-domains"]
```

**可用技能组：**

| 组名 | 技能数 | 描述 |
|------|--------|------|
| `rust-core` | 8 | 基础 Rust（所有权、并发、错误处理） |
| `rust-patterns` | 7 | 设计模式（领域建模、性能优化） |
| `rust-domains` | 7 | 领域特定（Web、CLI、金融科技、嵌入式） |
| `makepad` | 11 | Makepad UI 框架 |
| `dora` | 8 | Dora-rs 机器人框架 |
| `dora-hubs` | 9 | Dora Hub 集成 |

使用 `cowork config groups` 查看所有组及其技能。

### [triggers]

配置触发器冲突解决。

```toml
[triggers]
# 优先级顺序（第一个最高）
priority = ["dora-router", "rust-router", "makepad-router"]

[triggers.overrides]
# 显式触发器 -> 技能映射
"async" = "rust-router"
"widget" = "makepad-router"
"node" = "dora-router"
```

当多个技能匹配同一触发关键词时，优先级较高的技能获胜。覆盖提供显式映射，绕过优先级规则。

## CLI 命令

### 初始化

```bash
# 创建 Skills.toml 并自动检测已安装的 plugins/skills
cowork config init

# 自动检测会:
# 1. 扫描 ~/.claude/ 查找全局 plugins 和 skills
# 2. 扫描 .claude/ 查找项目 plugins 和 skills
# 3. 显示检测到的包并请求确认
# 4. 将选中的包添加到 Skills.toml

# 跳过自动检测（空配置）
cowork config init --no-detect

# 覆盖现有配置
cowork config init --force
```

**自动检测输出示例:**

```
Detected installed plugins/skills:

  Global ~/.claude/ (plugins):
    [1] rust-skills v1.2.0
    [2] makepad-skills v0.8.0 (symlink)

  Global ~/.claude/skills/ (skills):
    [3] memory-filesystem

  Project .claude/ (plugins):
    [4] dora-dev (symlink)

? Include in Skills.toml? [Y/n/select 1,2,3]: y
```

### 查看配置

```bash
# 显示当前配置
cowork config show

# 列出可用技能组
cowork config groups
```

### 管理依赖

```bash
# 从 GitHub 添加（安装到全局）
cowork config add rust-skills ZhangHanDong/rust-skills

# 添加到项目本地而非全局
cowork config add my-skills user/skills --local

# 添加指定技能
cowork config add tokio user/tokio-skills -s tokio-runtime -s tokio-sync

# 作为插件添加
cowork config add makepad user/makepad-skills --plugin

# 插件添加到项目本地
cowork config add dora user/dora-skills --plugin --local

# 添加为禁用（已安装但未启用）
cowork config add old-lib user/old --disabled

# 添加带 git ref
cowork config add pinned user/repo --ref v1.0.0

# 添加开发链接（用于测试的符号链接）
cowork config add dora-dev /path/to/dora-skills --dev

# 移除依赖
cowork config remove rust-skills

# 安装所有依赖
cowork config install
```

### 启用/禁用技能

```bash
# 启用技能组
cowork config enable rust-core makepad

# 启用单个技能
cowork config enable memory-filesystem

# 禁用技能或组
cowork config disable rust-domains domain-fintech
```

### 触发器配置

```bash
# 设置优先级顺序
cowork config priority dora-router rust-router makepad-router

# 覆盖特定触发器
cowork config override "async" rust-router
cowork config override "widget" makepad-router
```

### 生成输出

```bash
# 从配置生成 SKILLS.md
cowork config apply

# 生成到自定义路径
cowork config apply -o ./docs/SKILLS.md
```

### 生成动态路由器

```bash
# 根据已安装的 plugins 生成 cowork-router
cowork config router

# 生成路由器并带有自动触发的 hooks
cowork config router --hooks
```

路由器命令会:
1. 扫描已安装的 plugins 获取触发关键词
2. 在 `.claude/skills/cowork-router/` 生成动态 `cowork-router` skill
3. 可选择生成 `hooks.json` 用于基于关键词的自动触发

## 示例

### Rust Web 项目

```toml
[project]
name = "my-web-api"

[skills.global]
enabled = ["memory-filesystem"]

[skills.install]
rust = "ZhangHanDong/rust-skills"

[skills.groups]
enabled = ["rust-core", "rust-patterns"]

[triggers]
priority = ["rust-router"]

[triggers.overrides]
"async" = "rust-router"
"error" = "rust-router"
```

### Makepad UI 项目

```toml
[project]
name = "my-makepad-app"

[skills.global]
enabled = ["memory-filesystem", "best-skill-creator"]

[skills.groups]
enabled = ["rust-core", "makepad"]

[triggers]
priority = ["makepad-router", "rust-router"]

[triggers.overrides]
"widget" = "makepad-router"
"view" = "makepad-router"
"live_design" = "makepad-router"
```

### Dora 机器人项目

```toml
[project]
name = "my-robot"

[skills.groups]
enabled = ["rust-core", "dora", "dora-hubs"]

[triggers]
priority = ["dora-router", "rust-router"]

[triggers.overrides]
"node" = "dora-router"
"operator" = "dora-router"
"dataflow" = "dora-router"
```

### 多领域项目

```toml
[project]
name = "full-stack-robot"
description = "带 Makepad UI 的机器人"

[skills.global]
enabled = ["memory-filesystem"]

[skills.install]
rust = "ZhangHanDong/rust-skills"

[skills.groups]
enabled = ["rust-core", "makepad", "dora"]

[triggers]
priority = ["dora-router", "makepad-router", "rust-router"]

[triggers.overrides]
"widget" = "makepad-router"
"node" = "dora-router"
"async" = "rust-router"
```

## 技能解析顺序

当 Claude Code 处理查询时：

1. **检查触发器覆盖** - 显式映射优先
2. **匹配触发器** - 查找匹配查询关键词的技能
3. **应用优先级** - 优先级较高的技能获胜
4. **过滤启用/禁用** - 应用白名单/黑名单规则
5. **加载技能** - 执行选定的技能

## 最佳实践

1. **从最小开始** - 只启用你需要的技能
2. **使用组** - 比管理单个技能更容易
3. **设置优先级** - 避免模糊的触发器匹配
4. **使用覆盖** - 用于领域特定的关键词
5. **固定版本** - 生产环境使用 `ref` 确保稳定性
