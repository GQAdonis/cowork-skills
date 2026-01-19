# CoWork Skills

[English](./README.md) | [中文](./README-zh.md)

> マルチドメイン統合 Rust 開発アシスタント

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-Plugin-blue)](https://github.com/anthropics/claude-code)

## CoWork Skills とは？

**CoWork Skills** は複数の Rust ドメインスキルを統合した親プラグインです：

- **rust-skills** - Rust コア言語知識（所有権、並行性、エラー処理）
- **makepad-skills** - Makepad UI フレームワーク開発
- **dora-skills** - Dora-rs ロボティクスフレームワーク開発

## アーキテクチャ

```
cowork-skills/
├── skills/
│   ├── cowork-router/     # マスタールーター (ネイティブ)
│   ├── rust-router/       # → plugins/rust-skills/skills/ へのシンボリックリンク
│   ├── m01-ownership/     # → plugins/rust-skills/skills/ へのシンボリックリンク
│   └── ...                # 全サブプラグイン skills はシンボリックリンク経由
├── plugins/
│   ├── rust-skills/       # Git submodule - Rust コア
│   ├── makepad-skills/    # Git submodule - UI
│   └── dora-skills/       # Git submodule - ロボティクス
├── .claude/hooks/         # 統合 hooks
└── sync-skills.sh         # シンボリックリンク同期スクリプト
```

> **注意**: Claude Code はプラグインルートの `skills/` のみをロードします。サブプラグインの skills を含めるためにシンボリックリンクを使用しています。

## インストール

### クローン（サブモジュール含む）

```bash
git clone --recurse-submodules https://github.com/ZhangHanDong/cowork-skills.git
```

### Claude Code 起動

```bash
claude --plugin-dir /path/to/cowork-skills
```

### 権限設定

サンプル設定をプロジェクトにコピー：

```bash
cp /path/to/cowork-skills/.claude/settings.example.json .claude/settings.local.json
```

## 動作原理

```
ユーザー質問
     │
     ▼
┌─────────────────────────────────┐
│       cowork-router-hook        │
│    キーワードからドメイン検出      │
└─────────────────────────────────┘
     │
     ├─────────────┬─────────────┐
     ▼             ▼             ▼
┌─────────┐  ┌─────────┐  ┌─────────┐
│ Makepad │  │  Dora   │  │  Rust   │
│ルーター  │  │ルーター  │  │ルーター  │
└─────────┘  └─────────┘  └─────────┘
     │             │             │
     └─────────────┴─────────────┘
                   │
                   ▼
       ドメイン認識した回答
```

## ドメインスキル

| ドメイン | プラグイン | 説明 |
|----------|------------|------|
| **Rust コア** | rust-skills | 所有権、並行性、エラー処理、メタ認知フレームワーク |
| **UI 開発** | makepad-skills | Makepad widgets、views、live design |
| **ロボティクス** | dora-skills | Dora nodes、operators、dataflow |

## クロスドメイン質問

CoWork Skills は複数ドメインにまたがる質問を処理：

```
ユーザー: "Makepad widget で E0382 を処理するには？"

CoWork ルーティング:
├── メイン: Makepad (UI コンテキスト)
├── サブ: E0382 (Rust 所有権)
└── アクション: 両スキルをロードし、知識を組み合わせて回答
```

## サブモジュール更新

```bash
cd cowork-skills

# 全サブモジュール更新
git submodule update --remote

# 更新後にシンボリックリンクを再同期
./sync-skills.sh

# または特定のサブモジュール更新
cd plugins/rust-skills && git pull origin main
```

## 新しいサブプラグインの追加

```bash
cd cowork-skills

# 新しいプラグインを submodule として追加
git submodule add https://github.com/user/makepad-skills.git plugins/makepad-skills

# シンボリックリンクを同期
./sync-skills.sh
```

## 個別プラグイン使用

各プラグインは単独でも使用可能：

```bash
claude --plugin-dir /path/to/cowork-skills/plugins/rust-skills
claude --plugin-dir /path/to/cowork-skills/plugins/makepad-skills
```

## ライセンス

MIT ライセンス

## リンク

- **rust-skills**: https://github.com/ZhangHanDong/rust-skills
- **Issues**: https://github.com/ZhangHanDong/cowork-skills/issues
