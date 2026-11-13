#!/usr/bin/env bash
# エージェント設定の健全性チェック。正本（~/.agents）と各ハーネスの実ファイルの
# ドリフトを検出する。読み取り専用で、ファイルは変更しない。
#
# 使い方: ~/.agents/bin/check.sh [--quiet]
set -euo pipefail

AGENTS_HOME="${DSH_AGENTS_HOME:-$HOME/.agents}"
QUIET=0
[ "${1:-}" = "--quiet" ] && QUIET=1

say() { [ "$QUIET" -eq 1 ] || echo "$@"; }
fail=0

say "=== 1. 正本ファイルの存在 ==="
for f in AGENTS.md mcp.json agents bin/agents-sync.rs bin/mcp-sync.rs; do
  if [ -e "$AGENTS_HOME/$f" ]; then
    say "  OK   $f"
  else
    say "  NG   $f（正本がありません）"
    fail=1
  fi
done

say
say "=== 2. スキル形式（ディレクトリ名 = frontmatter name / description 必須） ==="
python3 - "$AGENTS_HOME" <<'PY' || fail=1
import os, re, sys
root = os.path.join(sys.argv[1], "skills")
bad = []
names = sorted(d for d in os.listdir(root) if os.path.isdir(os.path.join(root, d)))
for name in names:
    p = os.path.join(root, name, "SKILL.md")
    if not os.path.isfile(p):
        bad.append((name, "SKILL.md なし")); continue
    head = open(p, encoding="utf-8").read(4000)
    m = re.search(r"\A---\r?\n(.*?)\r?\n---", head, re.S)
    if not m:
        bad.append((name, "frontmatter なし")); continue
    fm = m.group(1)
    nm = re.search(r"^name:\s*['\"]?([^'\"\s]+)", fm, re.M)
    if not nm:
        bad.append((name, "name なし"))
    elif nm.group(1) != name:
        bad.append((name, f"name 不一致: {nm.group(1)}"))
    if not re.search(r"^description:\s*\S", fm, re.M) and "description: >-" not in fm:
        bad.append((name, "description なし"))
print(f"  {len(names)} スキル / 不正 {len(bad)}")
for n, why in bad:
    print(f"  NG   {n}: {why}")
sys.exit(1 if bad else 0)
PY

say
say "=== 3. サブエージェント定義の同期 ==="
if rust-script "$AGENTS_HOME/bin/agents-sync.rs" --check >/tmp/agents-check.log 2>&1; then
  say "  OK   $(tail -1 /tmp/agents-check.log)"
else
  say "  NG   ドリフトがあります:"
  sed 's/^/       /' /tmp/agents-check.log
  fail=1
fi

say
say "=== 4. DSH ホームのプリセット適用状態 ==="
if rust-script "$AGENTS_HOME/bin/agents-sync.rs" --check-dsh >/tmp/agents-dsh.log 2>&1; then
  say "  OK   $(tail -1 /tmp/agents-dsh.log)"
else
  say "  NG   未適用または不一致:"
  grep -E '^(OK|drift|未適用|agents-sync)' /tmp/agents-dsh.log | sed 's/^/       /'
  fail=1
fi

say
say "=== 5. MCP 単一ソースの同期（opencode） ==="
if rust-script "$AGENTS_HOME/bin/mcp-sync.rs" --check >/tmp/mcp-check.log 2>&1; then
  say "  OK   $(tail -1 /tmp/mcp-check.log)"
else
  say "  NG   ドリフトがあります:"
  grep -E '^(diff|mcp-sync)' /tmp/mcp-check.log | sed 's/^/       /'
  fail=1
fi

say
say "=== 6. MCP 単一ソースの同期（DSH プロファイル） ==="
if rust-script "$AGENTS_HOME/bin/mcp-sync.rs" --check-dsh >/tmp/mcp-dsh.log 2>&1; then
  say "  OK   $(tail -1 /tmp/mcp-dsh.log)"
else
  say "  NG   ドリフトがあります:"
  grep -E '^(diff|mcp-sync|skip)' /tmp/mcp-dsh.log | sed 's/^/       /'
  fail=1
fi

say
say "=== 7. 指示ファイルの markdownlint ==="
if command -v markdownlint-cli2 >/dev/null 2>&1; then
  CFG="$HOME/.config/opencode/.markdownlint-cli2.jsonc"
  if [ -f "$CFG" ]; then
    if (cd "$HOME/.config/opencode" && markdownlint-cli2 --config "$CFG" \
        "$AGENTS_HOME/AGENTS.md" "$AGENTS_HOME/README.md" \
        "$HOME/.config/opencode/AGENTS.md" >/tmp/md-lint.log 2>&1); then
      say "  OK   $(tail -1 /tmp/md-lint.log)"
    else
      say "  NG   markdownlint 違反:"
      grep -E 'error' /tmp/md-lint.log | head -20 | sed 's/^/       /'
      fail=1
    fi
  else
    say "  SKIP markdownlint 設定が見つかりません"
  fi
else
  say "  SKIP markdownlint-cli2 が未導入"
fi

say
if [ "$fail" -eq 0 ]; then
  say "結果: すべて一致（ドリフトなし）"
else
  say "結果: 差分または不正があります。上記を確認してください。"
fi
exit "$fail"
