---
type: Reference
title: "成果物の文書形式と GitHub issue 運用"
description: "OKF v0.3 フロントマター、markdownlint、index/log、命名規約、issue 起票条件"
tags:
  - okf
  - markdownlint
  - github-issue
---

## フロントマター（成果物ファイル）

- 先頭に [Open Knowledge Format (OKF) v0.3](https://github.com/open-knowledge-format/open-knowledge-format/blob/527e81aff5/SPEC.md) 準拠のYAMLフロントマターを置く（旧 GoogleCloudPlatform リポジトリは非推奨）。
- 必須キーは `type` のみ（未知の type も受容）。推奨キーは `title`・`description`・`tags`。任意キーは `resource`・`generated`・`verified`・`status`・`stale_after`・`superseded_by` および `x-<producer>-<name>` 形式の拡張キー。
- 日付・時刻は [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339) に準拠する。date-time（UTCオフセットまたは `Z`、例: `2026-09-19T09:00:00+09:00`）または full-date（`2026-09-19` = その日の UTC 00:00）を使う。オフセットなしの date-time は使わない。
- `generated.at` と `verified[].at` を比較し、`verified.at >= generated.at` の現在の検証イベントのみが信頼レベルに効く（v0.3 §5.3）。`status` は `draft` / `stable` / `deprecated` の3値で、未知の値は `stable` として扱わない。
- 「現在の日付」をメタデータに書く場合も自己認識で断定せず、`date` で確認する。

## 見出しとレイアウト

- markdownlint の警告を出さない（[markdownlint](https://github.com/DavidAnson/markdownlint) の MD013/MD025/MD041）。
- markdownlint はフロントマターの `title` をH1として扱う。`title` を含む成果物では本文の最初の見出しを `##` から始め、`##` → `###` → `####` の階段で構成する。本文に `#`（H1）を使わない。
- 予約ファイル `index.md` / `log.md` は例外で、フロントマターを持たず単一の `#` 見出しから始める。
- `index.md` は概念文書を収容する各ディレクトリに置き、配下へのリンクと `description` を列挙する。`log.md` は変更を加えたコミットで追記し、日付見出しは `## YYYY-MM-DD` で新しい順に並べる。自動生成する索引は領域をHTMLコメントで区切り、手動編集しない。
- 基本は [CommonMark](https://commonmark.org)。表・取り消し線・タスクリスト・コードブロックの言語指定が必要な場合は [GitHub Flavored Markdown](https://github.github.com/gfm/) に従い、衝突時はGFMを優先する。
- 文書を探すときは対象ディレクトリの `index.md` を先に読み、メタデータ検索（`type`・`tags`・`status`）で候補を絞る。索引やメタデータは候補選択の材料であり内容の保証ではないため、根拠にする前に本文を開く。全文検索とメタデータ検索を混同しない。古い索引を根拠にしない。

## GitHub issue の運用

- 個人リポジトリの issue は、セッションをまたぐプロジェクト固有の作業を追跡するためにエージェントが作成してよい。
- 作成する条件: 対象リポジトリで Issues が有効で既存 issue を検索して重複がない。作業内容が具体的で受け入れ条件と検証方法を書ける。セッション内で完了せず後続の作業または判断が必要。ユーザーが明示的に起票を求めた場合。
- 作成しない条件: 複数プロジェクト横断のタスク（グローバルTODOに置く）。一時的な進捗、思いつき、未検証の推測。秘密情報、認証情報、個人情報、非公開URLを含む内容。Issues が無効なリポジトリ（有効化はユーザー確認を得てから）。上流クローンなど自分が管理しないリポジトリ。
- 本文には背景、現状、期待する結果、受け入れ条件、検証方法を含める。公開リポジトリでは内部パス、ホスト名、ログを本文に含めない。
- ラベルは既存のものだけを使い、新規作成は確認を得る。コミットやPRから `Refs #<番号>` / `Closes #<番号>` で参照する。
- 完了時は検証結果をコメントしてから close する。未検証で close しない。起票と close は `gh issue create` / `gh issue close` を使い、一括操作はしない。
