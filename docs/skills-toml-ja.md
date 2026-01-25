# Skills.toml 設定ガイド

`Skills.toml` は、Rust プロジェクトの Cargo.toml に似た、Claude Code スキルのプロジェクトレベル設定ファイルです。ファイルはプロジェクトルートの `.cowork/Skills.toml` に保存されます。

## ワークフロー

```
┌─────────────────────────────────────────────────────────────────┐
│                     cowork config ワークフロー                    │
└─────────────────────────────────────────────────────────────────┘

1. cowork CLI をインストール
   $ cargo install --git https://github.com/anthropics/cowork-skills cli

2. プロジェクトで設定を初期化
   $ cd your-project
   $ cowork config init

   作成: your-project/.cowork/Skills.toml

3. スキルを設定（.cowork/Skills.toml を編集または CLI を使用）
   $ cowork config enable rust-core makepad
   $ cowork config add rust-skills ZhangHanDong/rust-skills

4. スキル依存関係をインストール
   $ cowork config install

5. 設定を適用（オプション - SKILLS.md を生成）
   $ cowork config apply

初期化後のディレクトリ構造：
   your-project/
   ├── .cowork/
   │   ├── Skills.toml      # スキル設定
   │   └── Skills.lock      # インストール済みスキルのロックファイル（自動生成）
   ├── skills/              # ローカルプロジェクトスキル（オプション）
   └── ...
```

## クイックスタート

```bash
# プロジェクトで .cowork/Skills.toml を初期化
cowork config init

# 現在の設定を表示
cowork config show

# スキル依存関係を追加
cowork config add rust-skills ZhangHanDong/rust-skills

# すべての依存関係をインストール
cowork config install
```

## ファイル位置

設定ファイルの保存場所：
- **プロジェクト設定**: `.cowork/Skills.toml`（プロジェクトルート）
- **ロックファイル**: `.cowork/Skills.lock`（自動生成）
- **グローバルスキル**: `~/.claude/skills/`（プロジェクト間で共有）
- **ローカルスキル**: `./skills/`（プロジェクト固有）

## Skills.lock

`Skills.lock` は `cowork config install` 実行時に自動生成されます。インストールされたスキル/プラグインの正確なバージョンと状態を記録します。

```toml
# このファイルは cowork により自動生成されます。手動で編集しないでください。

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

**ロックファイルのフィールド：**

| フィールド | 説明 |
|------------|------|
| `name` | パッケージ名 |
| `version` | plugin.json/package.json からのバージョン、またはインストールタイムスタンプ |
| `source` | `github`、`local`、または `dev` |
| `scope` | `global`（グローバル）または `project`（プロジェクト） |
| `type` | `plugin`、`skills`、または `symlink` |
| `install_path` | インストール場所 |
| `source_path` | 元のリポジトリまたはローカルパス |
| `git_sha` | Git コミット SHA（GitHub ソースの場合） |
| `installed_at` | インストールタイムスタンプ |
| `updated_at` | 最終更新タイムスタンプ |
| `enabled` | 有効かどうか |
| `skills` | インストールされたスキルのリスト |

## ファイル構造

```toml
# Skills.toml - プロジェクトスキル設定

[project]
name = "my-project"
description = "プロジェクトの説明"

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

## セクション説明

### [project]

プロジェクトメタデータ。

| フィールド | 型 | 説明 |
|------------|------|------|
| `name` | string | プロジェクト名（ディレクトリから自動検出） |
| `description` | string | オプションのプロジェクト説明 |

### [skills.global]

グローバルスキル（`~/.claude/skills/` から）を設定。

| フィールド | 型 | 説明 |
|------------|------|------|
| `enabled` | array | ホワイトリスト - これらのスキルのみロード |
| `disabled` | array | ブラックリスト - これらのスキルを除外 |

**ルール：**
- `enabled` が空の場合、すべてのグローバルスキルが利用可能
- `enabled` が設定されている場合、リストされたスキルのみロード
- `disabled` は常にスキルを除外（`enabled` にあっても）

### [skills.install]

GitHub またはローカルパスからのスキル依存関係を宣言。

**シンプル形式：**
```toml
[skills.install]
rust-skills = "ZhangHanDong/rust-skills"
```

**詳細形式：**
```toml
[skills.install]
# GitHub から特定のスキルをインストール
tokio = { repo = "user/tokio-skills", skills = ["tokio-runtime", "tokio-sync"] }

# ローカルパスから
local = { path = "../my-local-skills" }

# 特定バージョンに固定
pinned = { repo = "user/repo", ref = "v1.0.0" }

# ターゲットエージェント指定
multi = { repo = "user/repo", agents = ["claude-code", "cursor"] }
```

**プラグイン形式（リポジトリ構造を完全に保持）：**
```toml
[skills.install]
# プラグインとしてインストール - リポジトリ構造全体を保持
makepad = { repo = "user/makepad-skills", plugin = true }

# バージョン固定付きプラグイン
dora = { repo = "user/dora-skills", plugin = true, ref = "v2.0" }
```

**インストール場所（グローバル vs ローカル）：**
```toml
[skills.install]
# グローバル ~/.claude/skills/ にインストール（デフォルト）
rust-skills = "ZhangHanDong/rust-skills"

# プロジェクト .claude/skills/ にインストール
my-project = { repo = "user/skills", local = true }
my-plugin = { repo = "user/plugin", plugin = true, local = true }
```

**無効な依存関係（インストール済みだが有効化されていない）：**
```toml
[skills.install]
old-lib = { repo = "user/old", enabled = false }
```

| フィールド | 型 | 説明 |
|------------|------|------|
| `repo` | string | GitHub リポジトリ（user/repo） |
| `path` | string | ローカルファイルシステムパス |
| `skills` | array | インストールする特定のスキル（デフォルト：すべて） |
| `ref` | string | Git 参照（ブランチ、タグ、コミット） |
| `agents` | array | ターゲット AI エージェント |
| `plugin` | bool | プラグインとしてインストール（リポジトリ構造を完全に保持） |
| `local` | bool | グローバルではなくプロジェクトにインストール（デフォルト：false） |
| `enabled` | bool | この依存関係を有効/無効にする（デフォルト：true） |

### [skills.dev]

開発リンク（シンボリックリンク）- 開発中のローカルスキルをテストするため。

```toml
[skills.dev]
# シンプル形式：プロジェクト .claude/skills/ にリンク
my-skill = "/path/to/my-skill-project"

# 詳細形式：
dora-dev = { path = "/path/to/dora-skills" }
disabled-dev = { path = "/path/to/skills", enabled = false }

# プラグイン形式：.claude/skills/ ではなく .claude/<name>/ にリンク
dora-plugin = { path = "/path/to/dora-skills", plugin = true }
```

| フィールド | 型 | 説明 |
|------------|------|------|
| `path` | string | リンクするローカルパス |
| `local` | bool | プロジェクトにリンク（デフォルト：true） |
| `enabled` | bool | このリンクを有効/無効にする（デフォルト：true） |
| `plugin` | bool | プラグインとして `.claude/<name>/` にリンク（デフォルト：false） |

**リンク場所：**
- `plugin = false`（デフォルト）：`.claude/skills/<name>`（スキル）
- `plugin = true`：`.claude/<name>/`（プラグイン）

### [skills.groups]

事前定義されたスキルグループを有効化または無効化。

```toml
[skills.groups]
enabled = ["rust-core", "makepad"]
disabled = ["rust-domains"]
```

**利用可能なスキルグループ：**

| グループ | スキル数 | 説明 |
|----------|----------|------|
| `rust-core` | 8 | 基本 Rust（所有権、並行性、エラー処理） |
| `rust-patterns` | 7 | デザインパターン（ドメインモデリング、パフォーマンス） |
| `rust-domains` | 7 | ドメイン固有（Web、CLI、フィンテック、組み込み） |
| `makepad` | 11 | Makepad UI フレームワーク |
| `dora` | 8 | Dora-rs ロボティクスフレームワーク |
| `dora-hubs` | 9 | Dora Hub 統合 |

`cowork config groups` ですべてのグループとスキルを確認できます。

### [triggers]

トリガー競合解決を設定。

```toml
[triggers]
# 優先順位（最初が最高）
priority = ["dora-router", "rust-router", "makepad-router"]

[triggers.overrides]
# 明示的なトリガー -> スキルマッピング
"async" = "rust-router"
"widget" = "makepad-router"
"node" = "dora-router"
```

複数のスキルが同じトリガーキーワードにマッチする場合、優先度の高いスキルが選択されます。オーバーライドは優先順位ルールをバイパスする明示的なマッピングを提供します。

## CLI コマンド

### 初期化

```bash
# インストール済みの plugins/skills を自動検出して Skills.toml を作成
cowork config init

# 自動検出は以下を行います:
# 1. ~/.claude/ をスキャンしてグローバル plugins と skills を検出
# 2. .claude/ をスキャンしてプロジェクト plugins と skills を検出
# 3. 検出されたパッケージを表示して確認を求める
# 4. 選択されたパッケージを Skills.toml に追加

# 自動検出をスキップ（空の設定）
cowork config init --no-detect

# 既存の設定を上書き
cowork config init --force
```

**自動検出の出力例:**

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

### 設定表示

```bash
# 現在の設定を表示
cowork config show

# 利用可能なスキルグループを一覧
cowork config groups
```

### 依存関係管理

```bash
# GitHub から追加（グローバルにインストール）
cowork config add rust-skills ZhangHanDong/rust-skills

# グローバルではなくプロジェクトローカルに追加
cowork config add my-skills user/skills --local

# 特定のスキルを追加
cowork config add tokio user/tokio-skills -s tokio-runtime -s tokio-sync

# プラグインとして追加
cowork config add makepad user/makepad-skills --plugin

# プラグインをプロジェクトローカルに追加
cowork config add dora user/dora-skills --plugin --local

# 無効として追加（インストール済みだが有効化されていない）
cowork config add old-lib user/old --disabled

# git ref 付きで追加
cowork config add pinned user/repo --ref v1.0.0

# 開発リンクを追加（テスト用シンボリックリンク）
cowork config add dora-dev /path/to/dora-skills --dev

# 依存関係を削除
cowork config remove rust-skills

# すべての依存関係をインストール
cowork config install
```

### スキルの有効化/無効化

```bash
# スキルグループを有効化
cowork config enable rust-core makepad

# 個別のスキルを有効化
cowork config enable memory-filesystem

# スキルまたはグループを無効化
cowork config disable rust-domains domain-fintech
```

### トリガー設定

```bash
# 優先順位を設定
cowork config priority dora-router rust-router makepad-router

# 特定のトリガーをオーバーライド
cowork config override "async" rust-router
cowork config override "widget" makepad-router
```

### 出力生成

```bash
# 設定から SKILLS.md を生成
cowork config apply

# カスタムパスに生成
cowork config apply -o ./docs/SKILLS.md
```

### ダイナミックルーターを生成

```bash
# インストール済みプラグインに基づいて cowork-router を生成
cowork config router

# 自動トリガー用の hooks 付きでルーターを生成
cowork config router --hooks
```

ルーターコマンドは:
1. インストール済みプラグインからトリガーキーワードをスキャン
2. `.claude/skills/cowork-router/` に動的な `cowork-router` スキルを生成
3. オプションでキーワードに基づく自動トリガー用の `hooks.json` を生成

## 例

### Rust Web プロジェクト

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

### Makepad UI プロジェクト

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

### Dora ロボティクスプロジェクト

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

### マルチドメインプロジェクト

```toml
[project]
name = "full-stack-robot"
description = "Makepad UI を備えたロボット"

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

## スキル解決順序

Claude Code がクエリを処理する際：

1. **トリガーオーバーライドをチェック** - 明示的なマッピングが優先
2. **トリガーをマッチ** - クエリキーワードにマッチするスキルを検索
3. **優先順位を適用** - 優先度の高いスキルが選択される
4. **有効/無効をフィルター** - ホワイトリスト/ブラックリストルールを適用
5. **スキルをロード** - 選択されたスキルを実行

## ベストプラクティス

1. **最小限から始める** - 必要なスキルのみ有効化
2. **グループを使用** - 個別のスキル管理より簡単
3. **優先順位を設定** - 曖昧なトリガーマッチを回避
4. **オーバーライドを使用** - ドメイン固有のキーワードに
5. **バージョンを固定** - 本番環境では `ref` で安定性を確保
