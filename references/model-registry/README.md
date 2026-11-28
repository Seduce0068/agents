# モデルレジストリの運用

## 目的

モデルは入れ替わり、キャンペーンによって無料化やクオータ緩和が起きる。
特性・コスト・クオータをモデルごとに記録し、鮮度を明示して参照可能にする。
スキル本文にはモデル固有名も数値も固定しない。

## レコードの構成

`models/` に 1 モデル 1 ファイル（OKF 準拠のフロントマター付き）。

| セクション | 内容 |
| --- | --- |
| 登録情報 | モデル ID、表示名、プロバイダ、状態、初回確認日 |
| 特性 | 応答速度、推論・実装能力、コンテキスト長、安定性（値と出典と信頼度） |
| コスト / クオータ | スナップショット値、取得時刻、出典 |
| 調査 TODO | 未確認項目のリスト |
| 更新履歴 | いつ何を更新したか |

`status` は `active` / `candidate` / `archived` の 3 値。
未確認の値は推測で埋めず「未確認」と書く。

## ライフサイクル

### 新規モデルの検知

- `list_subagent_models` の結果と `index.md` を比較し、未登録なら `candidate` として記録する。
- カタログ（`list_models`）やコミュニティ情報に新モデルが出た場合も `candidate` にする。
- ステルスモデルは公式ドキュメントに載らないことがあるため、
  Exa などの Web 検索とコミュニティの投稿を重要な情報源として扱う。

### 調査

- `swarm-research` で特性・コスト・クオータを調べ、出典付きでレコードに追記する。
- コストは `get_usage` / `get_member_usage` のモデル別 `cost_usd` を実測値として使う。
- 残クオータは Go サブスクの [usage API](https://opencode.ai/docs/console/go/#usage)
  から取得する。現行パスは `GET https://opencode.ai/inference/go/v1/usage`
  （旧 `/zen/go/v1/usage` も存続）。Go と同じ API キーを Bearer で渡す。
- レスポンスは `usage.rolling`（5 時間）/`weekly`/`monthly` の
  `percent`（**使用済み** 0〜100）と `resetsAt`、`status`（`ok` / `rate-limited`）。
  5 時間 = 月間上限の 20%、週次 = 50%。ワークスペース全体の値でモデル別ではない。
- モデル別の月間上限は [models API](https://opencode.ai/docs/console/api/models)
  （`GET https://opencode.ai/inference/go/v1/models`）のメタデータで参照する。
- 認証なしは 401、Go 契約が無いキーは 403。Zen のプリペイド残高は API で読めず
  Console UI のみ（[issue #10448](https://github.com/anomalyco/opencode/issues/10448) 対応待ち）。
- 無料モデルのリクエスト上限は非公開のため、429 の発生状況を実測で記録する。
  取得時刻と出典を必ず添える。
- 出典には信頼度ラベル（公式 / 実測 / コミュニティ / 未確認）を付ける。

### 更新

- 選択時に `verified.at` が古い、429 が出た、コスト変化が見えた場合は再調査する。
- キャンペーン終了・無料化・クオータ緩和は新しいスナップショットで上書きし、
  古い値は更新履歴に残す。

### 退役（アーカイブ）

- 許可リストから消えた、または `not allowed for this Session` が出たら退役と判定する。
- レコードを `archive/` へ移動し、`archive/index.md` に 1 行追加する。
- `index.md` の active 表から外す。アーカイブ済みモデルは選択しない。
- 復活した場合は新規レコードとして再調査する（アーカイブをそのまま再利用しない）。

## Go usage スナップショット（認証済み実測）

取得時刻: 2026-11-28T15:49+09:00。認証にはローカルの `opencode` Personal
認証情報を使った。トークン自体はレジストリへ保存しない。

| ウィンドウ | status | percent（使用済み） | resetsAt |
| --- | --- | --- | --- |
| rolling（5h） | ok | 0 | 2026-11-28T09:09:35.000Z |
| weekly | ok | 25 | 2026-11-30T00:00:00.000Z |
| monthly | ok | 12 | 2026-12-24T14:40:38.000Z |

ワークスペース全体の値であり、モデル別の残量ではない。
`percent` は使用済みなので残量は `100 - percent` で求める。

`GET https://opencode.ai/inference/go/v1/models` はこの環境では無認証だと 403 になるため、
同じキーで呼び出した。公式メタデータ（コンテキスト長・価格・`go.monthly_usage`・
stealth 表示・学習利用の有無）を各レコードへ反映済み。

## スキルからの参照

swarm 系スキルは選択前に `index.md` を読み、active モデルの特性・コスト・クオータを確認する。
未記録・鮮度切れの場合は調査してから選ぶ。追加課金や API キーの新規登録は行わない。
