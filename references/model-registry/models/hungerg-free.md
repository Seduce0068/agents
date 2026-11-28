---
type: Reference
title: "hungerg-free"
description: "opencodeGO のステルス無料モデル。公式メタデータとコスト・クオータのスナップショット付き"
tags:
  - model
  - opencode-go
  - stealth
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
| モデル ID | `opencode-go/hungerg-free` |
| 表示名 | HungerG Free |
| プロバイダ | opencode-go |
| 状態 | active |
| 初回確認 | 2026-11-28 |

## 特性

| 観点 | 値 | 出典 | 信頼度 |
| --- | --- | --- | --- |
| 応答速度 | 未確認（ベンチ未実施） | - | 未確認 |
| 能力・特徴 | Stealth reasoning model, free for a limited time（期間限定無料） | models API | 公式 |
| 推論 | 対応（effort の選択肢は公開なし） | models API | 公式 |
| コンテキスト長 | 1,000,000 tokens | models API | 公式 |
| 出力上限 | 65,536 tokens | models API | 公式 |
| 学習利用 / ZDR | training: false / zdr: true | models API | 公式 |
| 安定性 | 未確認（失敗の有無は未計測） | - | 未確認 |

## コスト / クオータ（スナップショット）

| 項目 | 値 | 取得時刻 | 出典 |
| --- | --- | --- | --- |
| 公式価格（input / output） | 0 / 0 USD per 1M tokens | 2026-11-28T15:49+09:00 | models API |
| cost_usd（24h） | 0 | 2026-11-28T15:29+09:00 | `get_usage` 実測 |
| requests（24h） | 6 | 2026-11-28T15:29+09:00 | `get_usage` 実測 |
| Go 月間上限 | なし（monthly_usage: null） | 2026-11-28T15:49+09:00 | models API |
| 残クオータ | モデル別なし。workspace 全体は README の Go usage を参照 | 2026-11-28T15:49+09:00 | usage API |

## 調査 TODO

- [ ] ステルス-model の正体・ベンチをコミュニティ情報で裏取りする。
- [ ] 429 が出た時点の無料リクエスト上限を記録する。

## 更新履歴

- 2026-11-28 15:38: 初期レコード作成。
- 2026-11-28 15:50: 認証済み models API / usage API の値を追記。
