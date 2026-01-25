# CoWork Skills

[English](./README.md) | [中文](./README-zh.md)

> 16+ AI コーディングエージェント向けスキル管理 CLI ツール

[![Crates.io](https://img.shields.io/crates/v/cowork.svg)](https://crates.io/crates/cowork)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-Plugin-blue)](https://github.com/anthropics/claude-code)

## エージェントサポート状況

| エージェント | 状態 | 備考 |
|--------------|------|------|
| **Claude Code** | 完全テスト済み | 全機能検証済み |
| Cursor | コミュニティ | テストと貢献を歓迎 |
| Codex | コミュニティ | テストと貢献を歓迎 |
| GitHub Copilot | コミュニティ | テストと貢献を歓迎 |
| Windsurf | コミュニティ | テストと貢献を歓迎 |
| Goose | コミュニティ | テストと貢献を歓迎 |
| Amp | コミュニティ | テストと貢献を歓迎 |
| Roo | コミュニティ | テストと貢献を歓迎 |
| Kiro CLI | コミュニティ | テストと貢献を歓迎 |
| Gemini CLI | コミュニティ | テストと貢献を歓迎 |
| OpenCode | コミュニティ | テストと貢献を歓迎 |
| Antigravity | コミュニティ | テストと貢献を歓迎 |
| Clawdbot | コミュニティ | テストと貢献を歓迎 |
| Droid | コミュニティ | テストと貢献を歓迎 |
| Kilo | コミュニティ | テストと貢献を歓迎 |
| Trae | コミュニティ | テストと貢献を歓迎 |

> **注意:** CoWork Skills は Claude Code で完全にテストされています。他のエージェントのサポートは、文書化されたスキルディレクトリ規約に基づいています。**皆様のテストと貢献をお待ちしています！** これらのツールを使用している方は、互換性の検証と PR の提出にご協力ください。

## CoWork Skills とは？

**CoWork Skills** は複数のコーディングエージェントのスキルを管理する CLI ツール (`cowork` / `co`) を提供します：

- GitHub リポジトリから 16+ AI エージェントにスキルをインストール
- ソースコードからスキルを生成（Rust、TypeScript、Python）
- `Skills.toml` によるプロジェクトレベルの設定
- セキュリティ監査とチェックサム検証
- GitHub でスキルリポジトリを検索
- Claude Code マーケットプレイスプラグインを管理

## クイックスタート

### インストール

**ワンラインインストール（推奨）：**

```bash
curl -sSL https://raw.githubusercontent.com/ZhangHanDong/cowork-skills/main/install.sh | bash
```

**または手動インストール：**

```bash
# 方法 1：crates.io から
cargo install cowork

# 方法 2：ソースから
git clone https://github.com/ZhangHanDong/cowork-skills
cd cowork-skills/cli
cargo install --path .

# 組み込みスキルを初期化
cowork init
```

### スキルのインストール

```bash
# GitHub からインストール
cowork install user/repo

# 特定のスキルをインストール
cowork install user/repo -s skill1 -s skill2

# 特定のエージェントにインストール
cowork install user/repo -a claude-code -a cursor

# プラグインとしてインストール（リポジトリ構造を保持）
cowork install user/repo --plugin

# プロジェクトローカルにインストール (.claude/skills/)
cowork install user/repo --local

# 最新バージョンに更新
cowork install user/repo --update

# インストール済みリポジトリを一覧表示
cowork install --list
```

## CLI コマンド

| コマンド | 説明 |
|----------|------|
| `cowork init` | 組み込みスキルを ~/.claude/skills/ にインストール |
| `cowork install` | GitHub またはローカルパスからスキルをインストール |
| `cowork generate` | GitHub リポジトリからスキルを生成 |
| `cowork search` | GitHub でスキルリポジトリを検索 |
| `cowork plugins` | Claude Code マーケットプレイスプラグインを管理 |
| `cowork config` | プロジェクトレベルのスキル設定を管理 |
| `cowork list` | すべての利用可能なスキルを一覧表示 |
| `cowork status` | 現在の設定を表示 |
| `cowork doctor` | 設定の問題をチェック |
| `cowork test` | スキルのトリガーテストを生成・実行 |
| `cowork audit` | インストール済みスキルのセキュリティ監査 |
| `cowork verify` | インストール済みスキルのチェックサム検証 |

`co` を `cowork` の短いエイリアスとして使用できます。

## ソースコードからスキルを生成

ソースコードを解析して任意の GitHub リポジトリからスキルを生成：

```bash
# Rust スキルを生成
cowork generate tokio-rs/tokio --lang rust

# TypeScript スキルを生成
cowork generate vercel/next.js --lang typescript

# llms.txt のみを生成
cowork generate user/repo --llms-only -o ./output
```

### サポート言語

| 言語 | パーサー | 抽出内容 |
|------|----------|----------|
| Rust | `syn` | pub fn, struct, enum, trait, impl |
| TypeScript | `tree-sitter` | export function, class, interface, type |
| Python | `tree-sitter` | def, class（`_` プライベート項目を除く） |

## スキルを検索

```bash
# キーワードで検索
cowork search tokio

# GitHub トピックで検索
cowork search agent-skill --topic

# 詳細な結果を表示
cowork search rust-skills --verbose
```

## プラグインを管理

```bash
# マーケットプレイスプラグインを一覧表示
cowork plugins list

# プラグインステータスを表示
cowork plugins status

# プラグインをアンインストール
cowork plugins uninstall rust-skills

# プラグインを有効化/無効化
cowork plugins enable rust-skills
cowork plugins disable rust-skills
```

## 組み込みスキル

`cowork init` 実行後、以下のスキルがグローバルにインストールされます：

### memory-skills

CoALA 認知アーキテクチャに基づくメモリシステム。3つのコア機能を提供：

- **`/remember`** - 情報をメモリに保存（グローバル/プロジェクトスコープを自動検出）
- **`/recall`** - メモリから情報を検索・取得
- **`/summarize-session`** - 現在のセッションを要約しエピソード記憶に保存

メモリは3種類に分類：
- **意味記憶** - 事実、概念、ドメイン知識
- **エピソード記憶** - セッション要約、会話履歴
- **手続き記憶** - ワークフロー、パターン、操作ガイド

### cowork-guide

完全な CLI 使用ガイド。`cowork`、`Skills.toml`、または関連コマンドに言及すると自動的にトリガーされます。すべての CLI 機能のインラインドキュメントを提供。

### cowork-router

統合ルーター。キーワードとコンテキストに基づいて、質問を適切なインストール済みプラグイン/スキルに自動ルーティング。

### code-review

コードレビューアシスタント。`/review-pr` または `review PR` でトリガー。機能：
- GitHub API から PR diff を取得
- コード変更の問題を分析
- 重要度レベル付きの構造化フィードバックを提供
- ベストプラクティスに従った改善を提案

### github-generate

GitHub リポジトリからスキルを生成。`/github-generate` または `generate skill from repo` でトリガー。ソースコードを解析し、適切なトリガー付きのスキルファイルを作成。

### github-search

GitHub スキルリポジトリを検索。`/github-search` または `search for skills` でトリガー。`agent-skill` トピックまたはマッチするキーワードを持つリポジトリを検索。

### コマンド

```bash
# 利用可能な組み込みスキルを一覧表示
cowork init --list

# 特定の組み込みスキルをインストール
cowork init -s memory-skills -s cowork-guide

# プロジェクトローカルにインストール
cowork init --local

# 特定のスキルを削除
cowork init --remove memory-skills
```

## サポートエージェント

16以上のコーディングエージェントにスキルをインストール可能：

| エージェント | フラグ | エージェント | フラグ |
|--------------|--------|--------------|--------|
| Claude Code | `-a claude-code` | Amp | `-a amp` |
| Cursor | `-a cursor` | Antigravity | `-a antigravity` |
| Codex | `-a codex` | Clawdbot | `-a clawdbot` |
| GitHub Copilot | `-a github-copilot` | Droid | `-a droid` |
| Windsurf | `-a windsurf` | Gemini CLI | `-a gemini-cli` |
| Goose | `-a goose` | Kilo | `-a kilo` |
| Kiro CLI | `-a kiro-cli` | OpenCode | `-a opencode` |
| Roo | `-a roo` | Trae | `-a trae` |

```bash
# 複数のエージェントにインストール
cowork install user/repo -a claude-code -a cursor -a windsurf
```

## プロジェクト設定 (Skills.toml)

`Skills.toml` でプロジェクトレベルのスキル設定を管理：

```bash
# 設定を初期化（インストール済みの plugins/skills を自動検出）
cowork config init

# 自動検出をスキップ
cowork config init --no-detect

# 現在の設定を表示
cowork config show

# 依存関係を追加
cowork config add rust-skills ZhangHanDong/rust-skills
cowork config add makepad user/makepad-skills --plugin --local

# すべての依存関係をインストール
cowork config install

# ロックファイルを設定と同期
cowork config sync
cowork config sync --update  # リモートリポジトリも更新

# スキルまたはグループを有効化/無効化
cowork config enable rust-core
cowork config disable rust-domains

# トリガー優先順位を設定
cowork config priority dora-router rust-router

# 特定のトリガーをオーバーライド
cowork config override "async" rust-router

# 動的ルーターを生成
cowork config router
cowork config router --hooks    # 自動トリガーフック付き
cowork config router --analyze  # トリガー競合を分析

# 設定から SKILLS.md を生成
cowork config apply
```

詳細なドキュメントは [Skills.toml 設定ガイド](./docs/skills-toml-ja.md) を参照してください。

## スキルのテスト

スキルトリガーが正しく機能しているかテスト：

```bash
# トリガーテストレポートを生成
cowork test

# すべてのトリガーとそのスキルを一覧表示
cowork test triggers

# トリガー競合をチェック
cowork test --check-conflicts

# Claude を使用して実際のテストを実行
cowork test --run

# 特定のスキルをテスト
cowork test --filter "rust-*" --run

# スキルごとのトリガー数を制限
cowork test --run -n 5

# 出力形式
cowork test -o triggers.json --format json
cowork test -o triggers.yaml --format yaml
```

## セキュリティ

CoWork は悪意のあるスキルから保護するためのサプライチェーンセキュリティ機能を提供します：

### セキュリティ監査

```bash
# インストール済みスキルのセキュリティ問題をスキャン
cowork audit

# 特定の場所をスキャン
cowork audit --global           # ~/.claude/skills/ をスキャン
cowork audit --project          # .claude/skills/ をスキャン
cowork audit --plugins          # インストール済みプラグインをスキャン

# 詳細出力
cowork audit --verbose

# レポートをファイルに保存
cowork audit -o security-report.md --format markdown
cowork audit -o report.json --format json

# 問題を自動修正
cowork audit --fix
```

**検出機能：**
- 危険なパターン（`rm -rf`、`eval()`、`curl|sh`、`sudo`）
- プロンプトインジェクション攻撃
- 認証情報の漏洩（`API_KEY`、`PRIVATE KEY`、`password`）
- リスクレベル：SAFE、LOW、MEDIUM、HIGH、CRITICAL

### チェックサム検証

```bash
# Skills.lock に記録されたチェックサムに対してスキルを検証
cowork verify

# ロックファイルのチェックサムを更新
cowork verify --update

# 特定のスキルを検証
cowork verify rust-skills

# 詳細出力
cowork verify --verbose
```

### セキュリティ設定

`Skills.toml` に追加：

```toml
[security]
# 信頼する作者
trusted_authors = ["ZhangHanDong", "anthropics"]

# カスタムブロックパターン（正規表現）
blocked_patterns = ["dangerous-pattern"]

# スキャン時にスキップするパス（glob パターン）
skip_paths = [
    "**/docs/**",
    "**/examples/**",
    "**/tests/**",
]

# 信頼するマーケットプレイスプラグイン（スキャンをスキップ）
trusted_marketplaces = ["hookify", "rust-skills"]

# 高リスクスキルを自動拒否
auto_reject_high_risk = false
```

## ストレージロケーション

| 場所 | 用途 |
|------|------|
| `~/.cowork/repos/` | クローンした GitHub リポジトリ |
| `~/.claude/skills/` | グローバルスキルディレクトリ |
| `./skills/` | プロジェクトローカルスキル |

## 環境変数

| 変数 | 説明 |
|------|------|
| `GITHUB_TOKEN` | generate/search コマンドに必要 |

## ライセンス

MIT License

## リンク

- **rust-skills**: https://github.com/ZhangHanDong/rust-skills
- **Agent Skills 仕様**: https://agentskills.io
- **Skills マーケットプレイス**: https://skillsmp.com
- **llms.txt 仕様**: https://llmstxt.org
