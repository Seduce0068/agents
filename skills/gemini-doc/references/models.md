# モデル検証記録（Google / Gemini 無料枠）

**検証日**: 2026-11-11
**検証方法**: DSH資格情報ストアの `GOOGLE_API_KEY` を使い、
`https://generativelanguage.googleapis.com/v1beta` に直接アクセス（`models.list` + 各モデルへ `generateContent`）。
さらに DSH の `workflow` ツール（`agent()` に `provider: google` を指定）経由でも主要モデルの応答を確認。

> モデル構成・無料枠は変更される。再検証は `scripts/check_models.py`（後述）で行い、このファイルを更新する。

## 結論

- **既定モデルは `gemini-3.8-flash`**。日本語の作成・レビュー品質と速度のバランスが良い。
- 軽量処理は `gemini-3.5-flash-lite`、最速は `gemini-3.1-flash-lite`。
- Pro系・`gemini-4-argon`・deep-research系は**無料枠では応答しない**（429 QUOTA）。呼ばない。
- `gemini-2.5-pro` は廃止（404）。互換が必要な場合は `gemini-2.5-flash` を使う。
- 応答時間は測定条件で変わる。下表の「応答時間」は**API直接・1回計測**。
  DSHの `agent()` 経由はエージェント起動込みで数秒〜十数秒かかる。

## サブエージェント許可リスト（2026-11-11 ユーザー設定）

DSHのサブエージェントで使えるGoogleモデルは、ユーザーが設定した許可リストの11モデル。
**いずれも本検証でHTTP 200を確認済み**。

```
gemini-3.8-flash / gemini-3.7-flash / gemini-3.6-flash / gemini-3.5-flash /
gemini-3.5-flash-lite / gemini-3.1-flash-lite / gemini-3-flash-preview /
gemini-2.5-flash / gemini-2.5-flash-lite / gemini-flash-latest / gemini-flash-lite-latest
```

- 無料枠外（429）・廃止（404）・指示追従の弱いモデルは許可リストから除外済み。
- 許可リストはユーザー管理。スキル側で変更しない。
- **セッション開始後に許可リストを変更した場合、そのセッションには反映されない**
  （`child LLM route "google/..." is not allowed for this Session`）。
  `workflow` の `agent()` にフォールバックするか、新規セッションで実行する。

## 登録済み23モデルの結果（設定時点の記録）

`google` プロバイダに登録されていたモデルを1回ずつ実行した結果
（`maxOutputTokens: 2048`、プロンプト「Reply with exactly: OK」）。
「実体」は応答の `modelVersion`。エイリアスは別モデルにルーティングされる。

| モデルID | HTTP | 実体 (modelVersion) | 応答時間 | 判定 |
|---|---:|---|---:|---|
| `gemini-3.8-flash` | 200 | gemini-3.8-flash | 3.7s | **推奨（既定）** |
| `gemini-3.6-flash` | 200 | gemini-3.6-flash | 4.3s | 推奨 |
| `gemini-3.5-flash` | 200 | **gemini-3.6-flash** | 2.3s | エイリアス |
| `gemini-3.7-flash` | 200 | **gemini-3.8-flash** | 4.9s | エイリアス |
| `gemini-3.5-flash-lite` | 200 | gemini-3.5-flash-lite | 2.4s | 推奨（軽量） |
| `gemini-3.1-flash-lite` | 200 | gemini-3.1-flash-lite | 2.0s | 推奨（最速） |
| `gemini-3.1-flash-lite-preview` | 200 | **gemini-3.1-flash-lite** | 1.2s | エイリアス（許可リスト外） |
| `gemini-3-flash-preview` | 200 | gemini-3-flash-preview | 2.4s | 利用可 |
| `gemini-flash-latest` | 200 | **gemini-3.8-flash** | 3.6s | 非推奨（DSH経由で空応答の報告） |
| `gemini-flash-lite-latest` | 200 | **gemini-3.5-flash-lite** | 3.9s | 非推奨（実体が変わる） |
| `gemini-2.5-flash` | 200 | gemini-2.5-flash | 2.6s | 利用可（互換） |
| `gemini-2.5-flash-lite` | 200 | gemini-2.5-flash-lite | 2.7s | 利用可 |
| `gemini-robotics-er-2-preview` | 200 | gemini-robotics-er-2-preview | 2.3s | 用途外（許可リスト外） |
| `gemma-4-26b-a4b-it` | 200 | gemma-4-26b-a4b-it | 3.4s | 非推奨（指示追従が弱い） |
| `gemma-4-31b-it` | 200 | gemma-4-31b-it | 44.2s | 非推奨（指示追従が弱く低速） |
| `gemini-pro-latest` | 429 | — | 1.7s | **無料枠なし** |
| `gemini-3.1-pro-preview` | 429 | — | 0.9s | **無料枠なし** |
| `gemini-3.1-pro-preview-customtools` | 429 | — | 1.3s | **無料枠なし** |
| `gemini-4-argon` | 429 | — | 1.3s | **無料枠なし** |
| `deep-research-max-preview-04-2026` | 429 | — | 1.6s | **無料枠なし** |
| `deep-research-preview-04-2026` | 429 | — | 2.0s | **無料枠なし** |
| `gemini-2.5-computer-use-preview-10-2025` | 429 | — | 2.1s | **無料枠なし** |
| `gemini-2.5-pro` | 404 | — | 1.3s | **廃止**（`gemini-3.1-pro-preview` を案内されるが無料枠外） |

429 のメッセージは `RESOURCE_EXHAUSTED: You exceeded your current quota, please check your plan and billing details.`。
無料枠キーでは Pro系などが最初から利用不可のため、リトライしても通らない。

## 429 の見分け方

| メッセージ / 状況 | 意味 | 対応 |
|---|---|---|
| `check your plan and billing details` を含む | プラン上限（無料枠外） | モデル変更。リトライしない |
| `Retry-After` ヘッダあり / 短時間で解除 | 一時的なレート制限 | 指定秒待って1回だけ再試行 |
| 大量並列・連続呼び出しの直後 | 自分で RPM を超えた可能性 | 並列数・呼び出し回数を減らす |

無料枠の制限は「1回の429だけでは恒久的な無料枠外と断定できない」。
再検証時は 15秒以上あけて1回再試行し、それでも同じなら無料枠外と判断する（スクリプトはこの動作）。

## DSH経由の疎通確認（workflow / agent 上書き）

`workflow` ツールの `agent(prompt, { provider: 'google', model })` で7モデルが応答した（2026-11-11）。
表の秒数はエージェント起動込み。

| モデル | 結果 |
|---|---|
| `gemini-3.8-flash` | OK（約8.6秒） |
| `gemini-3.6-flash` | OK（約10.8秒） |
| `gemini-3.5-flash-lite` | OK（約9.0秒） |
| `gemini-3.1-flash-lite` | OK（約3.9秒） |
| `gemini-2.5-flash` | OK（約4.7秒） |
| `gemini-2.5-flash-lite` | OK（約3.7秒） |
| `gemini-flash-latest` | 応答は返るが最終テキストが空。明示モデルを使うこと |

## 無料枠の考え方

- 無料枠はAPIキー単位・モデル単位のレート/日次上限がある。大量に呼ぶと 429 になる。
- 429 時は 15秒以上待つ、`gemini-3.5-flash-lite` に落とす、章分割して呼ぶ回数を減らす。
- `models.list` は課金・クォータ消費なし。生成テスト（`--probe`）だけがクォータを消費する。

## 再検証スクリプト

前提: Python 3、`PyYAML`（`python3 -m pip install --user pyyaml` またはディストリ配布の `python3-yaml`）。
PyYAML が無い場合、スクリプトは分かりやすいエラーで終了する（環境変数キーによる `--list` は PyYAML 不要）。

キーの解決順:

1. 環境変数 `GOOGLE_API_KEY`
2. 環境変数 `GEMINI_API_KEY`
3. `--credential-store auto` を指定した場合のみ、DSH資格情報ストア
   （`$DSH_HOME/.credentials.yaml`、`~/.dsh-agentrouter/.credentials.yaml`、`~/.dsh/.credentials.yaml` の
   `refs.GOOGLE_API_KEY`）

**既定では資格情報ストアを読まない**（環境変数が無ければエラー終了）。
ストアを読む場合は `--credential-store auto` を明示する。読み取り時は警告を出し、キーは表示しない。

```bash
cd <skill>/scripts

# 一覧だけ（1リクエスト、クォータ消費なし）
python3 check_models.py --list --credential-store auto

# 登録済みモデルをすべて生成テスト（23リクエスト・約2分・無料枠を消費）
python3 check_models.py --probe --credential-store auto

# 特定モデルだけ
python3 check_models.py --probe --models gemini-3.8-flash,gemini-3.5-flash-lite --credential-store auto

# JSONでも保存（既存ファイルは --force なしでは上書きしない）
python3 check_models.py --probe --json result.json --credential-store auto
```

スクリプトの安全仕様:

- キーを**出力しない**（エラー時も `AIza...` 等のパターンをマスクする）。
- `--json` は既存ファイルを上書きしない（`--force` で明示許可）。
- 429/503 は 15秒待って1回再試行。SSL/接続エラーは指数バックオフで再試行。
- 設定ファイル解析に失敗した場合、既定モデルへ落ちたことを stderr に警告する。
