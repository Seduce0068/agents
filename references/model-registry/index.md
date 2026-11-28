# モデルレジストリ

最終更新: 2026-11-28T15:50:41+09:00（認証済み models API / usage API の実測を反映）

swarm 系スキルがサブエージェントのモデルを選ぶときの参照先。
運用手順は [README.md](README.md) を参照。

## active（選択可能）

DSH のサブエージェント設定に登録され、実際に選択できるモデル。
一覧の根拠は `list_subagent_models`（provider: `opencode-go`）。

| モデル | 状態 | verified.at | コスト（24h 実測） |
| --- | --- | --- | --- |
| [hy4-flash-free](models/hy4-flash-free.md) | active | 2026-11-28 | 0 USD |
| [beam-free](models/beam-free.md) | active | 2026-11-28 | 0 USD |
| [hungerg-free](models/hungerg-free.md) | active | 2026-11-28 | 0 USD |
| [deepseek-v4.1-flash](models/deepseek-v4.1-flash.md) | active | 2026-11-28 | 0.02410233 USD |
| [iceman-contributor-free](models/iceman-contributor-free.md) | active | 2026-11-28 | 未確認（使用実績なし） |

## candidate（未登録）

カタログやコミュニティ情報で見つかったが、サブエージェント設定に未登録のモデル。
ここからはモデルを選ばない。`swarm-research` で調査してレコードを作成してから評価する。

現時点で candidate レコードはなし。コンソールの OpenCode provider カタログ
（`list_models`）には多数の `-free` モデルが並ぶが、DSH のサブエージェント
許可リストは上記 5 件のみである。

## archived（非参照）

[アーカイブ一覧](archive/index.md) を参照。
アーカイブ済みのモデルは選択してはならない。
