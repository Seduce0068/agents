---
type: Reference
title: "deepseek-v4.1-flash"
description: "opencodeGO の課金モデル。公式価格・Go 月間上限とコスト実測を記録"
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
| モデル ID | `opencode-go/deepseek-v4.1-flash` |
| 表示名 | DeepSeek V4.1 Flash 1017 |
| プロバイダ | opencode-go |
| 状態 | active |
| 初回確認 | 2026-11-28 |

## 特性

| 観点 | 値 | 出典 | 信頼度 |
| --- | --- | --- | --- |
| 応答速度 | 未確認（ベンチ未実施） | - | 未確認 |
| 能力・特徴 | 10/17 更新。推論・コード・エージェント・画像理解を改善 | models API | 公式 |
| 推論 | 対応（effort: low / high / max） | models API | 公式 |
| コンテキスト長 | 1,000,000 tokens | models API | 公式 |
| 出力上限 | 384,000 tokens | models API | 公式 |
| 構造化出力ほか | 対応（temperature / top_p / top_k / system_updates） | models API | 公式 |
| 学習利用 / ZDR | training: false / zdr: true | models API | 公式 |
| 安定性 | 未確認（失敗の有無は未計測） | - | 未確認 |

## コスト / クオータ（スナップショット）

| 項目 | 値 | 取得時刻 | 出典 |
| --- | --- | --- | --- |
| 公式価格 input / output | 0.15 / 0.6 USD per 1M tokens | 2026-11-28T15:49+09:00 | models API |
| キャッシュ読み取り | 0.003 USD per 1M tokens | 2026-11-28T15:49+09:00 | models API |
| ピーク倍率 | 平日 01-04Z / 06-10Z は 2 倍 | 2026-11-28T15:49+09:00 | models API |
| Go 月間上限 | go: 100 / go-plus: 200 / go-teams: 105 USD | 2026-11-28T15:49+09:00 | models API |
| cost_usd（24h） | 0.02410233 | 2026-11-28T15:29+09:00 | `get_usage` 実測 |
| requests（24h） | 6 | 2026-11-28T15:29+09:00 | `get_usage` 実測 |
| 残クオータ | モデル別なし。workspace 全体は README の Go usage を参照 | 2026-11-28T15:49+09:00 | usage API |

単位あたりの価格はトークン数と時間帯に依存するため、24h 実測値から単価を算出して固定しない。

## 調査 TODO

- [ ] 応答速度と実品質をベンチまたは実測で埋める。
- [ ] キャンペーンによる価格変更・無料化が起きたらスナップショットを更新する。

## 更新履歴

- 2026-11-28 15:38: 初期レコード作成。
- 2026-11-28 15:50: 認証済み models API / usage API の値を追記。
