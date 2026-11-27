---
type: Reference
title: "ソフトウェア変更の標準検証ループ（詳細）"
description: "formatter から AI review までの検証手順、誤検知の扱い、rag-poc の失敗例"
tags:
  - verification
  - runtime-proof
---

## 標準検証ループ

コードを追加・変更した場合、対象リポジトリの `AGENTS.md`・`GEMINI.md`・README・CI設定を先に確認し、プロジェクト固有の検証方法を優先する。利用可能な範囲で次の順に実行する。

1. コード生成・変更を行い、既存の formatter を実行する。
2. compile / build / 型チェックを実行する。
3. プロジェクト標準の linter / static analysis を実行する。
4. 変更に対応する targeted tests を実行し、リスクに応じて suite 全体へ拡張する。
5. 最終差分をAI reviewし、正確性・回帰・セキュリティ・範囲外変更を確認する。
6. 採用した指摘を修正したら影響する検証を再実行し、終了時に結果を要約する。

- コマンド名を推測せず、プロジェクトで定義済みのコマンドを使う。
- 文書のみの変更では compile/lint/test を省略し、Markdown検証とリンク確認を行う。生成物や設定変更はそれぞれの専用検証を使う。
- AI review は補助的なゲートであり、compile/lint/test の代替にしない。
- AI review の指摘は仮説として扱い、現行コード・実際の制御フロー・テストで事実確認する。誤検知は根拠を記録して却下し、提案を無条件に適用しない。
- 性能・スケール上の指摘は、測定または再現根拠がない限り機能バグと区別する。
- 自動修正と再レビューは最大2回を目安にする。曖昧な指摘、設計判断が必要な指摘、破壊的操作、外部サービス依存、flaky test は無理に反復せず停止する。
- 既存の無関係な失敗を修正して差分を広げない。新規失敗と既存 baseline を区別し、検証不能・失敗は理由とともに報告する。
- リポジトリ由来コードの build/test 実行時は、秘密情報やホスト権限を不用意に露出させない。

## 具体的な失敗例（rag-poc）

テストで成果物パスを検証するときは、workspace 固有のディレクトリ名をハードコードせず `Path(__file__).resolve().parent` 等の基準パスと比較する。CI checkout のディレクトリ名はローカルと異なる。

2026-11-27 の rag-poc で `endswith("rag_poc/ragcore/...")` が CI の `/home/runner/work/rag-poc/rag-poc/...` に失敗した事例がある。同じセッションで `rust-gates.yml@v1` の input をハイフン区切りで渡し、`startup_failure` になった事例もある（詳細は [ci-delegation.md](ci-delegation.md)）。
