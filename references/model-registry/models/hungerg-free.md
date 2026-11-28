---
type: Reference
title: "hungerg-free"
description: "opencodeGO の無料モデル。特性は未確認、コストは 24h 実測で 0 USD"
tags:
  - model
  - opencode-go
generated:
  by: dsh-agent
  at: "2026-11-28T15:38:27+09:00"
verified:
  at: "2026-11-28T15:38:27+09:00"
status: active
---

## 登録情報

| 項目 | 値 |
| --- | --- |
| モデル ID | `opencode-go/hungerg-free` |
| 表示名 | HungerG Free |
| プロバイダ | opencode-go |
| 状態 | active |
| 初回確認 | 2026-11-28 |

## 特性

| 観点 | 値 | 出典 | 信頼度 |
| --- | --- | --- | --- |
| 応答速度 | 未確認 | - | 未確認 |
| 推論・実装能力 | 未確認 | - | 未確認 |
| コンテキスト長 | 未確認 | - | 未確認 |
| 安定性 | 未確認（失敗の有無は未計測） | - | 未確認 |

## コスト / クオータ（スナップショット）

| 項目 | 値 | 取得時刻 | 出典 |
| --- | --- | --- | --- |
| cost_usd（24h） | 0 | 2026-11-28T15:29+09:00 | `get_usage` 実測 |
| requests（24h） | 6 | 2026-11-28T15:29+09:00 | `get_usage` 実測 |
| 残クオータ | モデル別は未確認（Go usage は workspace 単位） | - | [usage API](https://opencode.ai/docs/console/go/#usage) |
| キャンペーン状態 | 無料（実測） | 2026-11-28T15:29+09:00 | `get_usage` 実測 |

## 調査 TODO

- [ ] 速度・能力・コンテキスト長を実測またはコミュニティ情報で埋める。
- [ ] 429 が出た時点の上限を記録する。

## 更新履歴

- 2026-11-28: 初期レコード作成（登録情報と `get_usage` のスナップショットのみ）。
