#!/usr/bin/env python3
"""Gemini（Google）無料枠モデルの疎通確認スクリプト。

DSH の google プロバイダ用 API キーで Gemini API に問い合わせる。
キーは絶対に出力しない（エラー時もマスクする）。

使い方:
  python3 check_models.py --list
  python3 check_models.py --list --credential-store auto   # DSH資格情報ストアを明示的に使う
  python3 check_models.py --probe                          # 登録モデルを生成テスト（無料枠を消費）
  python3 check_models.py --probe --models a,b --delay 4
  python3 check_models.py --probe --json out.json --force

キーの解決順:
  1. 環境変数 GOOGLE_API_KEY
  2. 環境変数 GEMINI_API_KEY
  3. --credential-store auto 指定時のみ DSH 資格情報ストア

既定では資格情報ストアを読まない（環境変数が無ければエラー終了）。
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import ssl
import sys
import time
import urllib.error
import urllib.request

BASE = "https://generativelanguage.googleapis.com/v1beta"

# DSH の google プロバイダ登録が見つからない場合のフォールバック
DEFAULT_MODELS = [
    "gemini-3.8-flash",
    "gemini-3.6-flash",
    "gemini-3.5-flash-lite",
    "gemini-3.1-flash-lite",
    "gemini-2.5-flash",
]

PROBE_BODY = {
    "contents": [{"role": "user", "parts": [{"text": "Reply with exactly: OK"}]}],
    "generationConfig": {"maxOutputTokens": 2048, "temperature": 0},
}

# エラー文字列に混入した秘密情報をマスクするパターン
_SECRET_PATTERNS = [
    (re.compile(r"AIza[0-9A-Za-z_\-]{10,}"), "AIza<REDACTED>"),
    (re.compile(r"(?i)((?:api[_-]?key|apikey|token|secret|password)\s*[:=]\s*)\S+"), r"\1<REDACTED>"),
    (re.compile(r"(?i)([?&](?:key|api_key|apikey|token)=)[^&\s]+"), r"\1<REDACTED>"),
    (re.compile(r"(?i)(Bearer\s+)[A-Za-z0-9._\-]+"), r"\1<REDACTED>"),
]


def mask(text: object) -> str:
    """秘密情報らしき文字列を伏せ字にする。"""
    value = str(text)
    for pattern, replacement in _SECRET_PATTERNS:
        value = pattern.sub(replacement, value)
    return value


def _yaml_module():
    """PyYAML を遅延 import する。無ければ None。"""
    try:
        import yaml
    except ImportError:
        print(
            "ERROR: PyYAML が見つかりません。\n"
            "  `python3 -m pip install --user pyyaml` を実行するか、\n"
            "  環境変数 GOOGLE_API_KEY を設定して --list を使用してください。",
            file=sys.stderr,
        )
        return None
    return yaml


def _yaml_loader(yaml):
    class Loader(yaml.SafeLoader):
        pass

    # DSH のプロファイルには !!js タグが含まれることがある
    Loader.add_multi_constructor(
        "tag:yaml.org,2002:js", lambda loader, suffix, node: loader.construct_scalar(node)
    )
    return Loader


def _candidate_stores():
    homes = [os.environ.get("DSH_HOME")]
    homes += [
        os.path.expanduser("~/.dsh-agentrouter"),
        os.path.expanduser("~/.dsh"),
    ]
    seen = set()
    for home in homes:
        if not home:
            continue
        path = os.path.join(home, ".credentials.yaml")
        if path in seen or not os.path.isfile(path):
            continue
        seen.add(path)
        yield path


def resolve_key(credential_store: str) -> tuple[str, str]:
    """(キー, キー取得元の説明) を返す。キー自体は表示しない。"""
    for env in ("GOOGLE_API_KEY", "GEMINI_API_KEY"):
        value = os.environ.get(env)
        if value:
            return value.strip(), f"env {env}"

    if not credential_store:
        return "", ""

    yaml = _yaml_module()
    if yaml is None:
        raise SystemExit(2)
    loader = _yaml_loader(yaml)

    if credential_store == "auto":
        paths = list(_candidate_stores())
    else:
        paths = [os.path.expanduser(credential_store)]

    for path in paths:
        if not os.path.isfile(path):
            continue
        try:
            with open(path, "r", encoding="utf-8") as fh:
                data = yaml.load(fh, Loader=loader)
        except Exception as exc:  # 破損ファイルは次の候補へ
            print(f"WARN: 資格情報ストアを読めません: {mask(exc)}", file=sys.stderr)
            continue
        refs = (data or {}).get("refs") or {}
        value = refs.get("GOOGLE_API_KEY")
        if value:
            print(
                f"NOTE: DSH資格情報ストアから GOOGLE_API_KEY を読みます（値は表示しません）: {path}",
                file=sys.stderr,
            )
            return str(value), f"DSH credential store ({path})"
    return "", ""


def configured_models(yaml) -> list[str]:
    """DSH プロファイル設定から google プロバイダの登録モデルを集める。

    NOTE: DSH の内部構造（profiles/*/cordis.patch.yml と llm-pi-ai エントリ）に依存する。
    取得できない場合は呼び出し側で既定モデルへフォールバックする。
    """
    if yaml is None:
        return []
    patterns = []
    for home in (os.environ.get("DSH_HOME"), "~/.dsh-agentrouter", "~/.dsh"):
        if not home:
            continue
        base = os.path.expanduser(home)
        patterns.append(os.path.join(base, "profiles", "*", "cordis.patch.yml"))

    loader = _yaml_loader(yaml)
    found: list[str] = []
    for pattern in patterns:
        for path in glob.glob(pattern):
            try:
                with open(path, "r", encoding="utf-8") as fh:
                    entries = yaml.load(fh, Loader=loader)
            except Exception:
                continue
            if not isinstance(entries, list):
                continue
            for entry in entries:
                if not isinstance(entry, dict) or entry.get("id") != "llm-pi-ai":
                    continue
                providers = ((entry.get("config") or {}).get("providers") or {})
                google = providers.get("google") or {}
                for model in google.get("models") or []:
                    mid = model.get("id")
                    if mid and mid not in found:
                        found.append(mid)
        if found:
            break
    return found


def api(path: str, key: str, *, method: str = "GET", body: dict | None = None,
        timeout: int = 90, retries: int = 3):
    """Gemini API 呼び出し。SSL/接続エラーは指数バックオフ（2,4,8秒）で再試行する。"""
    data = json.dumps(body).encode("utf-8") if body is not None else None
    last_error = ""
    for attempt in range(retries):
        request = urllib.request.Request(BASE + path, data=data, method=method)
        request.add_header("x-goog-api-key", key)
        request.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                return response.status, json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as exc:
            try:
                payload = json.loads(exc.read().decode("utf-8"))
            except Exception:
                payload = {}
            return exc.code, payload
        except ssl.SSLError as exc:
            last_error = f"SSL error: {exc}"
        except urllib.error.URLError as exc:
            if isinstance(exc.reason, ssl.SSLError):
                last_error = f"SSL error: {exc.reason}"
            else:
                last_error = f"connection error: {exc.reason}"
        time.sleep(min(2 ** attempt, 8))
    return 0, {"error": {"status": "NETWORK", "message": last_error or "unknown network error"}}


def extract_text(payload: dict) -> str:
    candidates = payload.get("candidates") or []
    if not candidates:
        return ""
    parts = (candidates[0].get("content") or {}).get("parts") or []
    return "".join(part.get("text", "") for part in parts).strip()


def probe_one(key: str, model: str, delay: float, retry_wait: float) -> dict:
    started = time.time()
    status, payload = api(f"/models/{model}:generateContent", key, method="POST", body=PROBE_BODY)
    if status in (429, 503):
        time.sleep(retry_wait)
        started = time.time()
        status, payload = api(f"/models/{model}:generateContent", key, method="POST", body=PROBE_BODY)
    elapsed = round(time.time() - started, 2)
    time.sleep(delay)

    result = {
        "model": model,
        "http": status,
        "seconds": elapsed,
        "modelVersion": "",
        "text": "",
        "error": "",
    }
    if status == 200:
        result["modelVersion"] = payload.get("modelVersion", "")
        result["text"] = mask(extract_text(payload)[:60])
    else:
        error = payload.get("error") or {}
        result["error"] = mask(f"{error.get('status', '')}: {str(error.get('message', ''))[:160]}")
    return result


def fmt_row(result: dict) -> str:
    if result["http"] == 200:
        body = result["text"] or "(empty)"
        version = result["modelVersion"] or "-"
        return f"{result['model']:<42} OK    {result['seconds']:>6}s  {version:<30} {body}"
    status = f"HTTP {result['http']}" if result["http"] else "ERROR"
    return f"{result['model']:<42} {status:<9} {result['seconds']:>6}s  {'':<30} {result['error']}"


def write_json(path: str, output: dict, force: bool) -> int:
    target = os.path.abspath(os.path.expanduser(path))
    if os.path.exists(target) and not force:
        print(f"ERROR: 出力先が既に存在します: {target}（上書きするなら --force）", file=sys.stderr)
        return 1
    try:
        with open(target, "w", encoding="utf-8") as fh:
            json.dump(output, fh, ensure_ascii=False, indent=2)
    except OSError as exc:
        print(f"ERROR: JSONを書き込めません: {mask(exc)}", file=sys.stderr)
        return 1
    print(f"\nJSON: {target}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Gemini 無料枠モデルの疎通確認")
    parser.add_argument("--list", action="store_true", help="利用可能モデルの一覧だけ表示（既定）")
    parser.add_argument("--probe", action="store_true", help="モデルへ生成リクエストを送って確認（無料枠を消費）")
    parser.add_argument("--models", default="", help="カンマ区切りのモデルID（--probe と併用）")
    parser.add_argument("--delay", type=float, default=4.0, help="リクエスト間隔（秒、既定4.0）")
    parser.add_argument("--retry-wait", type=float, default=15.0, help="429/503 時の待機秒（既定15）")
    parser.add_argument("--credential-store", default="",
                        help="DSH資格情報ストアを使う場合に 'auto' または .credentials.yaml のパスを指定")
    parser.add_argument("--json", dest="json_path", default="", help="結果をJSONで保存するパス")
    parser.add_argument("--force", action="store_true", help="--json の出力先を上書きする")
    args = parser.parse_args()

    key, source = resolve_key(args.credential_store)
    if not key:
        print("ERROR: Google API キーが見つかりません。", file=sys.stderr)
        print("  GOOGLE_API_KEY / GEMINI_API_KEY を設定するか、--credential-store auto を指定してください。",
              file=sys.stderr)
        return 2
    print(f"key source: {source}")

    output: dict = {"base": BASE, "generated": time.strftime("%Y-%m-%dT%H:%M:%S%z")}

    status, payload = api("/models?pageSize=1000", key)
    if status != 200:
        error = payload.get("error") or {}
        print(f"ERROR: models.list failed: HTTP {status} { mask(error.get('status', '')) } "
              f"{ mask(str(error.get('message', ''))[:200]) }", file=sys.stderr)
        return 1

    available = {}
    for model in payload.get("models", []):
        name = (model.get("name") or "").split("/")[-1]
        available[name] = model
    output["available"] = sorted(available)
    print(f"models.list: {len(available)} モデルが利用可能")

    if not args.probe:
        for name in sorted(available):
            methods = available[name].get("supportedGenerationMethods") or []
            print(f"  {name:<45} {','.join(methods)}")
        if args.json_path:
            return write_json(args.json_path, output, args.force)
        return 0

    models = [m.strip() for m in args.models.split(",") if m.strip()]
    if not models:
        yaml = None
        # configured_models が必要なときだけ PyYAML を要求する（環境変数キーの --list は PyYAML 不要）
        try:
            yaml = _yaml_module()
        except SystemExit:
            yaml = None
        configured = configured_models(yaml) if yaml is not None else []
        if not configured:
            print("WARN: DSHプロファイルから google モデル一覧を取得できませんでした。"
                  "既定モデルで実行します。", file=sys.stderr)
            configured = list(DEFAULT_MODELS)
        models = configured
    print(f"probe: {len(models)} モデル（間隔 {args.delay}s）")
    results = [probe_one(key, model, args.delay, args.retry_wait) for model in models]
    print()
    for result in results:
        print(fmt_row(result))
    output["results"] = results
    if args.json_path:
        return write_json(args.json_path, output, args.force)
    return 0


if __name__ == "__main__":
    sys.exit(main())
