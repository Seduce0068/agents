---
type: Reference
title: "hy4-flash-free"
description: "opencodeGO の無料モデル。公式メタデータとコスト・クオータのスナップショット付き"
tags:
  - model
  - opencode-go
generated:
  by: dsh-agent
  at: "2026-11-28T15:38:27+09:00"
verified:
  at: "2026-11-28T15:50:41+09:00"
status: active
---

## 登録情報

| 項目 | 値 |
| --- | --- |
| モデル ID | `opencode-go/hy4-flash-free` |
| 表示名 | Hy4 flash Free |
| プロバイダ | opencode-go |
| 状態 | active |
| 初回確認 | 2026-11-28 |

## 特性

| 観点 | 値 | 出典 | 信頼度 |
| --- | --- | --- | --- |
| 応答速度 | 未確認（ベンチ未実施） | - | 未確認 |
| 能力・特徴 | Tencent の低コスト Hy4 層。画像理解、日常エージェント向け | models API | 公式 |
| 推論 | 対応（effort: none / high） | models API | 公式 |
| コンテキスト長 | 1,048,576 tokens（input 983,040） | models API | 公式 |
| 出力上限 | 65,536 tokens | models API | 公式 |
| 学習利用 / ZDR | training: false / zdr: true | models API | 公式 |
| 安定性 | 未確認（失敗の有無は未計測） | - | 未確認 |

## コスト / クオータ（スナップショット）

| 項目 | 値 | 取得時刻 | 出典 |
| --- | --- | --- | --- |
| 公式価格（input / output） | 0 / 0 USD per 1M tokens | 2026-11-28T15:49+09:00 | models API |
| cost_usd（24h） | 0 | 2026-11-28T15:29+09:00 | `get_usage` 実測 |
| requests（24h） | 5344 | 2026-11-28T15:29+09:00 | `get_usage` 実測 |
| Go 月間上限 | なし（monthly_usage: null） | 2026-11-28T15:49+09:00 | models API |
| 残クオータ | モデル別なし。workspace 全体は README の Go usage を参照 | 2026-11-28T15:49+09:00 | usage API |

## 調査 TODO

- [ ] 応答速度と実品質をベンチまたは実測で埋める。
- [ ] 429 が出た時点の無料リクエスト上限を記録する。

## 更新履歴

- 2026-11-28 15:38: 初期レコード作成。
- 2026-11-28 15:50: 認証済み models API / usage API の値を追記。
