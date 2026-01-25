# CoWork 下一阶段功能规划

## 概述

四个主要任务：
1. 内置 Code Review Skill
2. Skills.toml 自动检测已安装插件
3. CoWork Guide Skill
4. 更新 README 文档

---

## Task 1: 内置 Code Review Skill

### 目标
创建一个综合代码审查 skill，利用 CLI 内置的 GitHub API 功能，支持：
- PR 代码审查
- Issues 分析
- Discussions 查阅
- 综合代码质量评估

### 文件结构
```
skills/code-review/
├── SKILL.md              # 主 skill 文件
└── references/
    ├── pr-review.md      # PR 审查指南
    ├── issue-analysis.md # Issue 分析模板
    └── review-checklist.md # 审查清单
```

### SKILL.md 设计
```markdown
# Code Review Skill

> Comprehensive code review using GitHub API

Triggers on: code review, PR review, review pull request,
审查代码, 代码评审, review issues, analyze PR

## Capabilities
1. Fetch and analyze PR changes
2. Review commit history
3. Check CI/CD status
4. Analyze related issues
5. Generate review comments

## Commands (via cowork CLI)
- `cowork review <owner/repo> --pr <number>`
- `cowork review <owner/repo> --issues`
- `cowork review <owner/repo> --discussions`
```

### CLI 命令扩展
```rust
// cli/src/commands/review.rs (新建)

/// Review options
pub struct ReviewOptions {
    pub repo: String,
    pub pr: Option<u64>,
    pub issues: bool,
    pub discussions: bool,
    pub since: Option<String>,  // 时间范围
    pub output: Option<PathBuf>,
}

/// 主要功能
pub async fn execute(options: ReviewOptions) -> Result<()> {
    // 1. 获取 PR 详情
    // 2. 获取 diff/changes
    // 3. 获取相关 issues
    // 4. 获取 CI 状态
    // 5. 生成审查报告
}
```

### 实施步骤
1. 创建 `cli/src/commands/review.rs`
2. 扩展 GitHub client 支持：
   - `get_pull_request()`
   - `get_pr_files()`
   - `get_pr_comments()`
   - `get_issues()`
   - `get_discussions()`
3. 创建 `skills/code-review/SKILL.md`
4. 在 main.rs 添加 Review 命令

---

## Task 2: Skills.toml 自动检测

### 目标
在 `cowork config init` 时自动检测并添加已安装的 plugins/skills

### 检测位置
```
全局:
  ~/.claude/skills/          # 全局 skills
  ~/.claude/*/               # 全局 plugins (排除 skills/)

项目:
  ./.claude/skills/          # 项目 skills
  ./.claude/*/               # 项目 plugins
```

### 检测逻辑
```rust
// cli/src/commands/config.rs

pub fn detect_installed_packages(project_root: &Path) -> DetectedPackages {
    let mut packages = DetectedPackages::default();

    // 1. 检测全局 plugins
    let global_claude = home_dir().join(".claude");
    for entry in read_dir(&global_claude) {
        if is_plugin(&entry) {
            packages.global_plugins.push(PluginInfo {
                name: entry.name(),
                version: get_version(&entry),
                path: entry.path(),
            });
        }
    }

    // 2. 检测全局 skills
    let global_skills = global_claude.join("skills");
    for skill in read_dir(&global_skills) {
        packages.global_skills.push(skill.name());
    }

    // 3. 检测项目 plugins/skills
    // ...

    packages
}

fn is_plugin(dir: &Path) -> bool {
    // 检查是否有 .claude-plugin/ 或 plugin.json
    dir.join(".claude-plugin").exists() ||
    dir.join("plugin.json").exists()
}
```

### 生成配置
```toml
# 自动生成的 Skills.toml

[skills.global]
enabled = ["memory-filesystem", "best-skill-creator"]  # 检测到的

[skills.install]
# 检测到的全局 plugins
rust-skills = { repo = "detected", plugin = true, local = false }
makepad-skills = { repo = "detected", plugin = true, local = false }

# 检测到的项目 plugins
# (已在 .claude/ 目录)
```

### 实施步骤
1. 添加 `detect_installed_packages()` 函数
2. 修改 `execute_init()` 调用检测
3. 生成带检测结果的 Skills.toml
4. 添加 `--no-detect` 选项跳过检测

---

## Task 3: CoWork Guide Skill

### 目标
创建全面的指南 skill，帮助 AI 和用户理解如何使用 cowork

### 文件结构
```
skills/cowork-guide/
├── SKILL.md                    # 主入口
└── references/
    ├── quick-start.md          # 快速开始
    ├── cli-commands.md         # CLI 命令参考
    ├── skills-toml-guide.md    # Skills.toml 配置指南
    ├── plugin-development.md   # 插件开发指南
    ├── skill-creation.md       # Skill 创建指南
    ├── troubleshooting.md      # 常见问题
    └── best-practices.md       # 最佳实践
```

### SKILL.md 设计
```markdown
# CoWork Guide

> Your guide to mastering CoWork Skills ecosystem

Triggers on: cowork help, how to use cowork, cowork guide,
cowork 帮助, 如何使用 cowork, skill 怎么用,
create skill, 创建 skill, install skill, 安装 skill

## Quick Start

### For Users
1. Install: `cargo install --git https://github.com/anthropics/cowork-skills cli`
2. Initialize: `cowork config init`
3. Install skills: `cowork config install`

### For Developers
1. Create skill: See [Skill Creation Guide](references/skill-creation.md)
2. Test locally: `cowork config add my-skill /path/to/skill --dev`
3. Publish: Push to GitHub

## CLI Commands Overview

| Command | Description |
|---------|-------------|
| `cowork init` | Install built-in skills |
| `cowork config init` | Initialize Skills.toml |
| `cowork config install` | Install dependencies |
| `cowork config sync` | Sync lock file |
| `cowork install` | Install from GitHub |

## Configuration

See [Skills.toml Guide](references/skills-toml-guide.md)

## Common Tasks

### Install a plugin
```bash
cowork config add rust-skills ZhangHanDong/rust-skills --plugin --local
cowork config install
```

### Enable/disable skills
```bash
cowork config enable rust-core makepad
cowork config disable rust-domains
```

### Update all plugins
```bash
cowork config sync --update
```
```

### 实施步骤
1. 创建 `skills/cowork-guide/SKILL.md`
2. 创建 references/ 下的所有指南文件
3. 将 `docs/skills-toml.md` 内容整合到 guide
4. 添加到内置 skills 列表

---

## Task 4: 更新 README

### 需要更新的文件
- `README.md` (English)
- `README-zh.md` (Chinese)
- `README-ja.md` (Japanese)

### 更新内容

#### 1. 新增功能说明
- Skills.toml / Skills.lock 配置系统
- `cowork config` 命令族
- `cowork config sync --update` 功能
- 自动检测已安装插件

#### 2. 更新 CLI 命令列表
```markdown
## CLI Commands

### Configuration Management
| Command | Description |
|---------|-------------|
| `cowork config init` | Initialize Skills.toml |
| `cowork config show` | Show current configuration |
| `cowork config add` | Add dependency |
| `cowork config install` | Install all dependencies |
| `cowork config sync` | Sync lock file |
| `cowork config sync -u` | Sync + update remotes |

### Installation
| Command | Description |
|---------|-------------|
| `cowork install` | Install from GitHub |
| `cowork init` | Install built-in skills |
```

#### 3. 添加配置文件说明
- Skills.toml 格式简介
- Skills.lock 说明
- 链接到详细文档

#### 4. 更新架构图
```
┌─────────────────────────────────────────────────────────────┐
│                        cowork CLI                            │
├─────────────────────────────────────────────────────────────┤
│  config   │  install  │  review   │  generate  │  search   │
├───────────┴───────────┴───────────┴────────────┴───────────┤
│                     Skills.toml                              │
│                     Skills.lock                              │
├─────────────────────────────────────────────────────────────┤
│  Global (~/.claude/)  │  Project (.claude/)  │  Dev Links   │
└─────────────────────────────────────────────────────────────┘
```

---

## 实施顺序

### Phase 1: 基础设施 (Task 2 + 4)
1. ✅ 实现 `detect_installed_packages()`
2. ✅ 更新 `cowork config init`
3. ✅ 更新 README 文档

### Phase 2: Guide Skill (Task 3)
1. ✅ 创建 cowork-guide skill 结构
2. ✅ 编写所有参考文档
3. ✅ 集成到内置 skills

### Phase 3: Code Review (Task 1)
1. ✅ 扩展 GitHub API client
2. ✅ 实现 review 命令
3. ✅ 创建 code-review skill
4. ✅ 测试和文档

---

## 预计工作量

| Task | 估计文件数 | 复杂度 |
|------|-----------|--------|
| Task 1: Code Review | 5-8 | 高 |
| Task 2: Auto Detect | 2-3 | 中 |
| Task 3: Guide Skill | 8-10 | 中 |
| Task 4: README | 3 | 低 |

**建议顺序**: Task 2 → Task 4 → Task 3 → Task 1

---

## 问题待确认

1. **Code Review**:
   - 是否需要支持 GitLab/Gitee?
   - 审查报告输出格式 (Markdown/JSON)?

2. **Auto Detect**:
   - 检测到未知来源的 plugin 如何处理?
   - 是否需要用户确认?

3. **Guide Skill**:
   - 是否需要多语言版本?
   - 是否包含视频/动图教程链接?
