---
type: Reference
title: "シークレットスキャン（Betterleaks）の運用"
description: "pre-commit 主軸の検出、CI ゲート、画像 OCR、履歴改変後の再走査"
tags:
  - secrets
  - betterleaks
  - pre-commit
---

## 基本方針

機微情報（秘密情報・個人情報）が混じっていないか確認する作業では、**Betterleaks を必ず使用する**。grep や目視だけで「該当なし」と判断してはならない。リポジトリの公開前、コミット・push 前の確認でも同じこと。確認は `betterleaks git` により履歴全体を対象に行い、HEAD だけで判断しない。Betterleaks は秘密情報の検出が主目的のため、住所・病歴・家族構成などの個人情報は目視確認も併用する。

## 導入

- **pre-commit を主軸**にする。pre-push は補助として追加してもよいが、単独運用は避ける。シークレットがコミット履歴に入る前に検出・ブロックできるため。
- pre-push 単独では検出時点で既にコミット履歴に残っており、完全な削除には `git rebase` / `git filter-repo` 等の履歴書き換えが必要になる。公開後は共同作業者への影響とキーのローテーションが発生する。
- ローカルフックは `SKIP=betterleaks` / `git commit --no-verify` で回避できるため、**CI でのスキャンを最終ゲートとして必ず併設**する。
- 設定例（[v1.x 公式 README](https://github.com/betterleaks/betterleaks/blob/v1.x/README.md) 準拠）: `.pre-commit-config.yaml` に `repo: https://github.com/betterleaks/betterleaks` と `id: betterleaks` を置き、`pre-commit install` と `pre-commit autoupdate` で運用する。2026-11-22 時点の安定版は v1.10.1 で、`main` ブランチは v2 開発中のため `rev` は v1 タグにピン留めする。
- Betterleaks は Gitleaks の開発陣（原作者を含む）が保守する後継ツール。詳細は[公式リポジトリ](https://github.com/betterleaks/betterleaks)を参照する。

## 画像と履歴改変

- **Betterleaks は画像内テキストを走査しない**。スクリーンショットに実アカウント名・ホスト名・履歴・マウント先が写る場合、`betterleaks git` が hit zero でも漏洩は残る。画像を含むリポジトリでは `tesseract <image> stdout -l eng` 等の OCR を別途走査し、実識別子（アカウント名、内部ホスト、IP、ローカルパス）を検出すること。OCR は日本語のみの画像では精度が落ちるため、必要なら `eng+jpn` や目視を併用する。
- **`git filter-repo` 等の履歴改変後は `.betterleaksignore` の fingerprint が無効になる**（コミットハッシュが変わるため）。履歴改変後は必ず `betterleaks git` を再走査し、現在の fingerprint で allowlist を再生成して hit zero を確認する。fingerprint 行にダミー接続 URI などの文字列が残ると ignore ファイル自身が再検出されるため、コメントはマスク済みの表現に留める。
