# ~/.agents — ハーネス非依存のエージェント設定（正本）

複数のコーディングエージェント（DSH / opencode / 将来追加するハーネス）で共有する
設定の正本。ハーネス固有の設定は各ハーネス側に残し、ここには**ハーネスに依存しない
部分だけ**を置く。

## 構成

| パス | 内容 | 読み手 |
|---|---|---|
| `AGENTS.md` | 全ハーネス共通の指示（言語・安全・環境・検証・ブリッジ・文書形式・Rust・MCP原則） | DSH: 自動 / opencode: `instructions` 設定 |
| `skills/<name>/SKILL.md` | Agent Skills 標準のスキル | DSH / opencode: 自動 |
| `agents/<name>.md` | サブエージェント定義（中立形式: `name` / `description` / `access` + 本文） | 生成器経由で各ハーネスへ |
| `mcp.json` | MCP サーバの単一ソース（ハーネス別のネイティブ表現を含む） | 検証・生成の基準 |
| `harness/<name>/` | ハーネス固有の上書き（例: `opencode/agents.json`） | 生成器が読む |
| `bin/agents-sync.rs` | サブエージェント定義の同期・検証 | 手動実行 |
| `bin/mcp-sync.rs` | MCP 設定の同期・検証 | 手動実行 |
| `bin/check.sh` | 全体のドリフト検査（読み取り専用） | 定期点検 |
| `generated/` | 生成物（各ハーネスに貼る断片） | 参照用 |
| `hooks/hooks.json` | Codex 系フック設定（ハーネス固有・参考） | Codex |

## サブエージェントの同期

正本は `agents/<name>.md`。frontmatter の `access` が権限クラス（`read-only` /
`tests-only`）を表し、実際の権限表とモデルは `harness/opencode/agents.json` が持つ。

```bash
rust-script ~/.agents/bin/agents-sync.rs --list           # 一覧
rust-script ~/.agents/bin/agents-sync.rs --check          # ドリフト検査（exit 1 で差分）
rust-script ~/.agents/bin/agents-sync.rs --emit-opencode  # generated/ へ出力
rust-script ~/.agents/bin/agents-sync.rs --apply          # opencode の agents/ を更新
rust-script ~/.agents/bin/agents-sync.rs --emit-dsh       # DSH の agent-preset パッチを生成
rust-script ~/.agents/bin/agents-sync.rs --check-dsh      # DSH ホームへの適用状態を検査
```

### DSH への適用（2026-11-14 適用済み）

ホーム直下の `$DSH_HOME/cordis.patch.yml` は**全プロファイルへ適用される**ユーザー層。
`--emit-dsh` の出力をそのまま置く。

```bash
cp ~/.agents/generated/dsh/agent-presets.patch.yml ~/.dsh-agentrouter/cordis.patch.yml
cp ~/.agents/generated/dsh/agent-presets.patch.yml ~/.dsh/cordis.patch.yml
DSH_HOME=~/.dsh dsh web --dump-config | grep -c 'id: preset-'   # 9（5体 + 標準4）
```

サービスへ反映するには再起動する（自分のセッションが動いている側は避ける）。

```bash
systemctl --user restart dsh-web.service              # :3080
systemctl --user restart dsh-web-agentrouter.service  # :3081（このGUI）
```

**プリセットの選択について**: 現在 `allowedAgentPresets` は未設定のため、DSH では
プリセットは**登録されるが選択UIには既定の継承**となる。`spawn_teammate`（Agent Teams）は
プリセットを選べない。`subagent` ツールで選択するには agent-team 系を無効化して
`tool-subagent` を有効化する必要があり、これは Agent Teams を置き換える変更。
生成物の末尾に手順をコメントとして同梱してある。

## MCP の単一ソース

正本は `mcp.json` の `servers.<name>.harnesses.<harness>`。`{"supported": false, "reason": ...}`
はそのハーネスで使わない（使えない）ことを意味する。

```bash
rust-script ~/.agents/bin/mcp-sync.rs --list        # 一覧
rust-script ~/.agents/bin/mcp-sync.rs --check       # opencode.jsonc との一致検査
rust-script ~/.agents/bin/mcp-sync.rs --check-dsh   # DSH プロファイルとの一致検査
rust-script ~/.agents/bin/mcp-sync.rs --emit-opencode --emit-dsh
rust-script ~/.agents/bin/mcp-sync.rs --apply       # opencode.jsonc の mcp ブロックを置換
```

`--apply` は `opencode.jsonc` の `mcp` ブロック**だけ**を置換し、他の記述とコメントは
保持する。書き込み前に生成結果を JSON として構文検証し、不正なら中止する（fail closed）。
DSH 側は `--emit-dsh` の出力を各プロファイルの `cordis.patch.yml` へ追記する。

## 全体検査

```bash
~/.agents/bin/check.sh          # 正本・スキル形式・エージェント同期・MCP同期・markdownlint
```

## 追跡方針

- `skills/` のうち**個人作成のものだけ**を `.gitignore` の whitelist で追跡する。
- 外部から導入したスキル（Azure/Codex バンドル等）は `.skill-lock.json` と
  `.copilot-for-azure-skills-manifest.json` が管理するため追跡しない。
- 秘密情報は置かない。API キー等は各ハーネスの秘密管理（`{env:}` / `{file:}` 参照）に置く。

## 各ハーネスとの接続（2026-11-12 時点）

- **DSH**（0.2.2-rc.1）
  - `~/.agents/AGENTS.md` と `~/.agents/skills` を既定で読む（追加設定不要）。
  - `DSH_HOME` は `~/.dsh`（dsh-web :3080）と `~/.dsh-agentrouter`（agentrouter :3081）の
    2つがあるが、どちらも同じ共有設定を読む。ハーネス側の旧 symlink
    （`~/.dsh/AGENTS.md`、`~/.dsh/skills`）は 2026-11-12 に撤去した。
- **opencode**（v2.2.6）
  - `~/.agents/skills` を標準サポート（追加設定不要）。
  - 指示は `~/.config/opencode/opencode.jsonc` の
    `"instructions": ["~/.agents/AGENTS.md"]` で共有する。
  - opencode 固有の指示は `~/.config/opencode/AGENTS.md` に残す（環境事実・ブリッジ節は
    重複を削除し、このリポジトリを正とした）。

## 検証

```bash
# スキルの形式チェック（ディレクトリ名と frontmatter name の一致・description の有無）
python3 - <<'PY'
import os, re, sys
root = os.path.expanduser("~/.agents/skills")
bad = []
for name in sorted(os.listdir(root)):
    p = os.path.join(root, name, "SKILL.md")
    if not os.path.isfile(p):
        bad.append((name, "SKILL.md なし")); continue
    head = open(p, encoding="utf-8").read(2000)
    m = re.search(r"^---\n(.*?)\n---", head, re.S)
    if not m: bad.append((name, "frontmatter なし")); continue
    fm = m.group(1)
    nm = re.search(r"^name:\s*(\S+)", fm, re.M)
    ds = re.search(r"^description:", fm, re.M)
    if not nm or nm.group(1) != name: bad.append((name, f"name不一致: {nm.group(1) if nm else 'なし'}"))
    if not ds: bad.append((name, "description なし"))
print("skills:", len(os.listdir(root)), "invalid:", len(bad))
for b in bad: print(" -", b)
sys.exit(1 if bad else 0)
PY
```

## 残課題（Phase 4 候補）

- DSH への agent-preset パッチ適用（`generated/dsh/agent-presets.patch.yml` を
  `~/.dsh-agentrouter/cordis.patch.yml` へ反映）。モデル選択で使う場合は各 preset 行の
  `tool-subagent` に `allowedAgentPresets` を追加する必要がある。
- MCP の実設定を生成物から流し込む運用（現在は検証のみで、`opencode.jsonc` が正）。
- `~/.dsh` と `~/.dsh-agentrouter` の 2 ホーム統合の判断。

## Phase 計画

1. **Phase 1（2026-11-12 実施）**: スキルを `~/.agents/skills` に一本化、共通指示を
   `~/.agents/AGENTS.md` に集約、opencode へ `instructions` 追加、環境事実とブリッジ節の
   重複を排除、`~/.dsh` の旧 symlink を撤去。
2. **Phase 2（2026-11-13 実施）**: 共通コア 28KB / opencode 固有 9.5KB に分割。ハーネス固有語は
   opencode 側へ移動し、チャット応答の frontmatter/markdownlint 適用を成果物限定に修正。
   markdownlint 0 issues。
3. **Phase 3（2026-11-14 実施）**: サブエージェント定義を `agents/` に中立形式で集約し、
   `bin/agents-sync.rs` で opencode の agents を生成（往復一致を確認）。MCP を `mcp.json` に
   単一ソース化し `bin/mcp-sync.rs` で両ハーネスを検証（15 サーバ）。DSH の agent-preset
   パッチ生成も実装し、`--dump-config` で本文の完全一致を確認。`bin/check.sh` で統合検査。

## 変更履歴

- 2026-11-12: 新規作成。`~/.config/opencode/skills` の個人スキル11本
  （ai-selection / cross-review / doc-create / okf / opencode-environment-report /
  policy-law / review-flow / tech-selection / test-spec / typst / userscript-validator）を移管。
  壊れていた `skills/okf` を修復（SKILL.md + scripts を統合）。
- 2026-11-13: Phase 2。`AGENTS.md` を共通コア（28KB）と opencode 固有（9.5KB）に分割。
  合計 37.5KB（旧 44.5KB）。
- 2026-11-14: Phase 3。サブエージェント正本 `agents/`（5体）、MCP 単一ソース `mcp.json`
  （15サーバ）、生成・検証 `bin/`（agents-sync.rs / mcp-sync.rs / check.sh）を追加。
  opencode の agents を生成物へ置換（markdownlint 0 issues）。
