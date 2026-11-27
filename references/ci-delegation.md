---
type: Reference
title: "個人リポジトリの CI 委譲（rust-gates.yml）"
description: "共通ワークフローへの委譲、input の snake_case、タグピンと self-test"
tags:
  - ci
  - rust
  - github-actions
---

## 委譲先

個人リポジトリ（`github.com/Seduce0068/*`）の Rust プロジェクトは、CIを共通の再利用可能ワークフローへ委譲し、ゲートをリポジトリごとに複製しない。

- 提供側はリポジトリ `Seduce0068/workflows` の `rust-gates.yml`。ゲートは fmt → check → test → clippy → extra_gates で、標準検証ループと同じ構成にする。
- 呼び出し側は `.github/workflows/ci.yml` に `uses:` の1ジョブを置く。`permissions: contents: read` と `concurrency` は呼び出し側に置く。
- 入力（apt依存、wasmターゲット、追加ゲート等）と記述例は提供側READMEを参照する。

## 既知の失敗例

- `rust-gates.yml@v1` の input は snake_case（`working_directory` / `extra_gates` / `clippy_args` 等）。`manifest-path` / `extra-gates` のようなハイフン区切りは未定義 input となり `startup_failure` になる（2026-11-27 rag-poc で実証）。呼び出し前に `on.workflow_call.inputs` と渡すキーを照合する。ローカルの YAML 構文チェックでは検出できない。

## 運用

- 参照はタグ `v1` にピン留めする。互換を保つ更新はタグを付け替え、破壊的変更は `v2` を切る。
- 共通ワークフロー自体の変更は `self-test.yml`（fixtures/hello の回帰テスト）で検証してからタグを動かす。
- CIの変更は、まず共通ワークフロー側で対応できないか検討する。
- 提供側 `Seduce0068/workflows` は public（2026-10-17公開）のため、public・private を問わずどのリポジトリからも参照できる。public リポジトリも委譲できる（例: OpenCodeRust）。
