# ~/.agents — ハーネス非依存のエージェント設定（正本）

複数のコーディングエージェント（DSH / opencode / 将来追加するハーネス）で共有する
設定の正本。ハーネス固有の設定は各ハーネス側に残し、ここには**ハーネスに依存しない
部分だけ**を置く。

## 構成

| パス | 内容 | 読み手 |
|---|---|---|
| `AGENTS.md` | 全ハーネス共通の指示（言語・安全・環境・検証・ブリッジ・文書形式・Rust・MCP原則） | DSH: 自動 / opencode: `instructions` 設定 |
| `skills/<name>/SKILL.md` | Agent Skills 標準のスキル | DSH / opencode: 自動 |
| `hooks/hooks.json` | Codex 系フック設定（ハーネス固有・参考） | Codex |
| `.skill-lock.json` | 外部スキルの導入来歴（skills CLI 管理・非追跡） | skills CLI |

`agents/`（サブエージェント定義）と `mcp.json`（MCP 単一ソース）は Phase 2/3 で追加する。

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

## Phase 計画

1. **Phase 1（2026-11-12 実施）**: スキルを `~/.agents/skills` に一本化、共通指示を
   `~/.agents/AGENTS.md` に集約、opencode へ `instructions` 追加、環境事実とブリッジ節の
   重複を排除、`~/.dsh` の旧 symlink を撤去。
2. **Phase 2（2026-11-13 実施）**: 共通コア 28KB / opencode 固有 9.5KB に分割。ハーネス固有語は
   opencode 側へ移動し、チャット応答の frontmatter/markdownlint 適用を成果物限定に修正。
   markdownlint 0 issues。
3. **Phase 3**: サブエージェント定義（`agents/`）の共有、MCP 単一ソース化（`mcp.json`）、
   生成・検証スクリプト（`bin/`）の整備。

## 変更履歴

- 2026-11-12: 新規作成。`~/.config/opencode/skills` の個人スキル11本
  （ai-selection / cross-review / doc-create / okf / opencode-environment-report /
  policy-law / review-flow / tech-selection / test-spec / typst / userscript-validator）を移管。
  壊れていた `skills/okf` を修復（SKILL.md + scripts を統合）。
- 2026-11-13: Phase 2。`AGENTS.md` を共通コア（28KB）と opencode 固有（9.5KB）に分割。
  合計 37.5KB（旧 44.5KB）。
