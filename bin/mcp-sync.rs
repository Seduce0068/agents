//! ```cargo
//! [dependencies]
//! serde_json = "1"
//! ```
//!
//! MCP 設定の同期・検証ツール。
//!
//! 正本: `~/.agents/mcp.json`
//!   servers.<name>.harnesses.<harness> が各ハーネスのネイティブ表現を持つ。
//!   ハーネス側が `{"supported": false}` の場合は「そのハーネスでは使わない」を意味する。
//!
//! 使い方:
//!   mcp-sync.rs --check           opencode.jsonc と mcp.json の一致を検査（差分があれば exit 1）
//!   mcp-sync.rs --check-dsh       DSH プロファイルの MCP 行と mcp.json の一致を検査
//!   mcp-sync.rs --emit-opencode   opencode 用の MCP ブロック断片を generated/ へ出力
//!   mcp-sync.rs --emit-dsh        DSH 用の MCP パッチ断片を generated/ へ出力
//!   mcp-sync.rs --list            正本のサーバ一覧を表示

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn home() -> PathBuf {
    PathBuf::from(env::var("HOME").expect("HOME is required"))
}

fn agents_root() -> PathBuf {
    match env::var("DSH_AGENTS_HOME") {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v),
        _ => home().join(".agents"),
    }
}

/// JSONC（コメントと末尾カンマ）を素の JSON へ落とす。
fn strip_jsonc(src: &str) -> String {
    let bytes: Vec<char> = src.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    let mut in_str = false;
    let mut esc = false;
    while i < bytes.len() {
        let c = bytes[i];
        let n = if i + 1 < bytes.len() { bytes[i + 1] } else { '\0' };
        if in_str {
            out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && n == '/' {
            while i < bytes.len() && bytes[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && n == '*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == '*' && bytes[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        out.push(c);
        i += 1;
    }
    // 末尾カンマを除去
    let mut res: Vec<char> = Vec::with_capacity(out.len());
    let mut s2 = false;
    let mut e2 = false;
    let mut j = 0usize;
    while j < out.len() {
        let c = out[j];
        if s2 {
            res.push(c);
            if e2 {
                e2 = false;
            } else if c == '\\' {
                e2 = true;
            } else if c == '"' {
                s2 = false;
            }
            j += 1;
            continue;
        }
        if c == '"' {
            s2 = true;
            res.push(c);
            j += 1;
            continue;
        }
        if c == ',' {
            let mut k = j + 1;
            while k < out.len() && out[k].is_whitespace() {
                k += 1;
            }
            if k < out.len() && (out[k] == '}' || out[k] == ']') {
                j += 1;
                continue;
            }
        }
        res.push(c);
        j += 1;
    }
    res.into_iter().collect()
}

/// 正本のエントリ。
#[derive(Debug, Clone)]
struct Server {
    name: String,
    description: String,
    /// 正本の `$comment`（元設定の詳細コメント。生成時にそのまま出力する）。
    comment: Option<String>,
    enabled: bool,
    opencode: Option<serde_json::Value>,
    opencode_unsupported: Option<String>,
    dsh: Option<serde_json::Value>,
    dsh_unsupported: Option<String>,
}

fn load_servers() -> Result<Vec<Server>, String> {
    let path = agents_root().join("mcp.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let servers = doc
        .get("servers")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "servers オブジェクトがありません".to_string())?;
    let mut out = Vec::new();
    for (name, entry) in servers {
        let harnesses = entry.get("harnesses").and_then(|v| v.as_object());
        let pick = |key: &str| -> (Option<serde_json::Value>, Option<String>) {
            let Some(h) = harnesses.and_then(|h| h.get(key)) else {
                return (None, None);
            };
            if let Some(reason) = h.get("reason").and_then(|v| v.as_str()) {
                return (None, Some(reason.to_string()));
            }
            (Some(h.clone()), None)
        };
        let (opencode, opencode_unsupported) = pick("opencode");
        let (dsh, dsh_unsupported) = pick("dsh");
        out.push(Server {
            name: name.clone(),
            description: entry
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            enabled: entry.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
            comment: entry.get("$comment").and_then(|c| c.as_array()).map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            }),
            opencode,
            opencode_unsupported,
            dsh,
            dsh_unsupported,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// 正本の opencode 表現を実際の opencode.jsonc の表現へ正規化する。
fn normalize_opencode(v: &serde_json::Value) -> serde_json::Value {
    let mut obj = v.as_object().cloned().unwrap_or_default();
    // 正本側の付帯キーは比較対象から外す
    obj.remove("profiles");
    obj.remove("supported");
    obj.remove("reason");
    obj.remove("$comment");
    // disabled はサーバ単位の enabled から導出するため、定義側の値は使わない
    obj.remove("disabled");
    serde_json::Value::Object(obj)
}

fn normalize_actual_opencode(v: &serde_json::Value) -> serde_json::Value {
    let mut obj = v.as_object().cloned().unwrap_or_default();
    obj.remove("disabled");
    serde_json::Value::Object(obj)
}

fn check_opencode(servers: &[Server], jsonc_path: &Path) -> Result<usize, String> {
    let text = fs::read_to_string(jsonc_path).map_err(|e| format!("{}: {e}", jsonc_path.display()))?;
    let doc: serde_json::Value = serde_json::from_str(&strip_jsonc(&text))
        .map_err(|e| format!("{}: JSON として解釈できません: {e}", jsonc_path.display()))?;
    let actual = doc
        .get("mcp")
        .and_then(|v| v.get("servers"))
        .and_then(|v| v.as_object())
        .ok_or_else(|| "mcp.servers がありません".to_string())?;

    let mut problems = 0usize;
    for s in servers {
        match (&s.opencode, actual.get(&s.name)) {
            (Some(expected), Some(found)) => {
                let want = normalize_opencode(expected);
                let got = normalize_actual_opencode(found);
                if want != got {
                    println!("diff: {} の定義が正本と一致しません", s.name);
                    let w = serde_json::to_string(&want).unwrap_or_default();
                    let g = serde_json::to_string(&got).unwrap_or_default();
                    println!("  expected: {}", truncate(&w, 300));
                    println!("  actual  : {}", truncate(&g, 300));
                    problems += 1;
                }
                let want_disabled = !s.enabled;
                let got_disabled = found.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
                if want_disabled != got_disabled {
                    println!(
                        "diff: {} の enabled 状態が不一致（正本 enabled={} / 実ファイル disabled={}）",
                        s.name, s.enabled, got_disabled
                    );
                    problems += 1;
                }
            }
            (Some(_), None) => {
                println!("diff: {} が opencode.jsonc にありません", s.name);
                problems += 1;
            }
            (None, Some(_)) => {
                if let Some(reason) = &s.opencode_unsupported {
                    println!("diff: {} は opencode では未対応のはず（{reason}）だが実ファイルにあります", s.name);
                    problems += 1;
                }
            }
            (None, None) => {}
        }
    }
    // mcp.json にないサーバが実ファイルにある場合
    let known: std::collections::BTreeSet<&String> = servers.iter().map(|s| &s.name).collect();
    for name in actual.keys() {
        if !known.contains(name) {
            println!("diff: {} が実ファイルにありますが正本（mcp.json）にありません", name);
            problems += 1;
        }
    }
    Ok(problems)
}

/// DSH プロファイルの cordis.patch.yml から `serverName` を抽出する。
fn dsh_server_names(patch: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(patch).map_err(|e| format!("{}: {e}", patch.display()))?;
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("serverName:") {
            out.push(rest.trim().to_string());
        }
    }
    Ok(out)
}

fn check_dsh(servers: &[Server], profiles: &[(&str, PathBuf)]) -> Result<usize, String> {
    let mut problems = 0usize;
    for (profile, path) in profiles {
        if !path.exists() {
            println!("skip: {profile} の {path:?} が存在しません");
            continue;
        }
        let found = dsh_server_names(path)?;
        for s in servers {
            let expected = match &s.dsh {
                Some(v) => {
                    let scoped = v
                        .get("profiles")
                        .and_then(|p| p.as_array())
                        .map(|a| a.iter().any(|x| x.as_str() == Some(profile)))
                        .unwrap_or(true);
                    if !scoped {
                        continue;
                    }
                    v.get("serverName")
                        .and_then(|x| x.as_str())
                        .map(str::to_string)
                        .unwrap_or_else(|| s.name.clone())
                }
                None => continue,
            };
            if !found.contains(&expected) {
                println!("diff: {profile} に DSH サーバ {expected} がありません（正本: {}）", s.name);
                problems += 1;
            }
        }
        for name in &found {
            let known = servers.iter().any(|s| {
                s.dsh
                    .as_ref()
                    .and_then(|v| v.get("serverName").and_then(|x| x.as_str()))
                    .map(|x| x == name)
                    .unwrap_or(false)
            });
            if !known {
                println!("diff: {profile} に {name} がありますが正本（mcp.json）にありません");
                problems += 1;
            }
        }
    }
    Ok(problems)
}

/// opencode.jsonc の `mcp` ブロック全体を生成する（コメント付き・インデント付き）。
fn render_opencode_block(servers: &[Server]) -> String {
    let mut out = String::new();
    out.push_str("  \"mcp\": {\n");
    out.push_str("    // このブロックは ~/.agents/bin/mcp-sync.rs --apply が生成する。直接編集しない。\n");
    out.push_str("    // 正本: ~/.agents/mcp.json\n");
    out.push_str("    \"servers\": {\n");
    let last_enabled = servers
        .iter()
        .filter(|s| s.opencode.is_some())
        .map(|s| s.name.clone())
        .next_back();
    for s in servers {
        let Some(v) = &s.opencode else { continue };
        let is_last = last_enabled.as_deref() == Some(s.name.as_str());
        // 説明コメント（正本の $comment があればそれを使う）
        let comment = s
            .comment
            .clone()
            .unwrap_or_else(|| s.description.clone());
        // 正本の行構造を尊重する（改行は保持、長い行だけ折り返す）
        for logical in comment.split('\n') {
            for line in wrap_comment(logical, 86) {
                out.push_str(&format!("      // {line}\n"));
            }
        }
        // enabled から disabled を導出して注入する（正本は enabled のみを持つ）
        let mut entry = normalize_opencode(v);
        if let Some(obj) = entry.as_object_mut() {
            obj.insert("disabled".into(), serde_json::Value::Bool(!s.enabled));
        }
        out.push_str(&format!(
            "      \"{}\": {}{}\n",
            s.name,
            render_json_value(&entry),
            if is_last { "" } else { "," }
        ));
    }
    out.push_str("    }\n");
    out.push_str("  }");
    out
}

/// 長いコメントを折り返す（日本語幅は概算2として数える）。
fn wrap_comment(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut w = 0usize;
    for ch in text.chars() {
        let cw = if ch.is_ascii() { 1 } else { 2 };
        if w + cw > width && !current.is_empty() {
            lines.push(current.clone());
            current.clear();
            w = 0;
        }
        current.push(ch);
        w += cw;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// JSON 値を 6 スペース基準で整形する（サーバ本体）。
fn render_json_value(v: &serde_json::Value) -> String {
    let pretty = serde_json::to_string_pretty(v).unwrap_or_default();
    let mut out = String::new();
    for (i, line) in pretty.lines().enumerate() {
        if i == 0 {
            out.push_str(line);
        } else {
            out.push_str(&format!("\n      {line}"));
        }
    }
    out
}

fn emit_dsh(servers: &[Server]) -> String {
    let mut out = String::new();
    out.push_str(
        "# このパッチは ~/.agents/bin/mcp-sync.rs --emit-dsh が生成する。直接編集しない。\n\
         # 正本: ~/.agents/mcp.json\n\n",
    );
    for s in servers {
        let Some(v) = &s.dsh else { continue };
        let server_name = v.get("serverName").and_then(|x| x.as_str()).unwrap_or(&s.name);
        out.push_str("- insert:\n");
        out.push_str(&format!("    - id: mcp-{server_name}\n"));
        out.push_str("      name: '@deepseek-ai/dsh-mcp-client'\n");
        out.push_str("      config:\n");
        out.push_str(&format!("        serverName: {server_name}\n"));
        let transport = v.get("transport").and_then(|x| x.as_str()).unwrap_or("streamable-http");
        out.push_str(&format!("        transport: {transport}\n"));
        if let Some(url) = v.get("url").and_then(|x| x.as_str()) {
            out.push_str(&format!("        url: {url}\n"));
        }
        if let Some(command) = v.get("command").and_then(|x| x.as_str()) {
            out.push_str(&format!("        command: {command}\n"));
        }
        if let Some(headers) = v.get("headers").and_then(|x| x.as_object()) {
            out.push_str("        headers:\n");
            for (k, val) in headers {
                out.push_str(&format!("          {k}: {}\n", val.as_str().unwrap_or("")));
            }
        }
        if let Some(t) = v.get("toolCallTimeoutMs").and_then(|x| x.as_u64()) {
            out.push_str(&format!("        toolCallTimeoutMs: {t}\n"));
        }
        if v.get("failOnStartupError").and_then(|x| x.as_bool()) == Some(false) {
            out.push_str("        failOnStartupError: false\n");
        }
        out.push('\n');
    }
    out
}

/// opencode.jsonc の `mcp` ブロックだけを生成物で置換する。他の記述は保持する。
fn apply_opencode(_servers: &[Server], jsonc_path: &Path) -> Result<bool, String> {
    let src = fs::read_to_string(jsonc_path).map_err(|e| format!("{}: {e}", jsonc_path.display()))?;
    let lines: Vec<&str> = src.split('\n').collect();
    let start = lines
        .iter()
        .position(|l| l.trim() == "\"mcp\": {")
        .ok_or_else(|| "opencode.jsonc に \"mcp\": { が見つかりません".to_string())?;
    let mut depth = 0i32;
    let mut end = None;
    for (i, line) in lines.iter().enumerate().skip(start) {
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        if i > start && depth == 0 {
            end = Some(i);
            break;
        }
    }
    let end = end.ok_or_else(|| "mcp ブロックの終端が見つかりません".to_string())?;
    let replacement = render_opencode_block(_servers);
    let mut out_lines: Vec<String> = Vec::new();
    out_lines.extend(lines[..start].iter().map(|s| s.to_string()));
    out_lines.extend(replacement.split('\n').map(|s| s.to_string()));
    // 元の末尾がカンマ付きだった場合は合わせる
    let original_last = lines[end].trim_end();
    if original_last.ends_with(',') {
        if let Some(last) = out_lines.last_mut() {
            if !last.ends_with(',') {
                last.push(',');
            }
        }
    }
    out_lines.extend(lines[end + 1..].iter().map(|s| s.to_string()));
    let updated = out_lines.join("\n");
    if updated == src {
        return Ok(false);
    }
    // 置換前に構文検証（素の JSON として解釈できるか）
    serde_json::from_str::<serde_json::Value>(&strip_jsonc(&updated))
        .map_err(|e| format!("生成した JSONC が不正なため書き込みを中止: {e}"))?;
    fs::write(jsonc_path, updated).map_err(|e| format!("{}: {e}", jsonc_path.display()))?;
    Ok(true)
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let t: String = s.chars().take(n).collect();
        format!("{t}…")
    }
}

fn write_if_changed(path: &Path, content: &str) -> Result<bool, String> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == content {
            return Ok(false);
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, content).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("--check");
    let servers = match load_servers() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("mcp-sync: {e}");
            return ExitCode::FAILURE;
        }
    };

    match mode {
        "--list" => {
            for s in &servers {
                let oc = if s.opencode.is_some() {
                    "opencode:yes"
                } else {
                    "opencode:no "
                };
                let ds = if s.dsh.is_some() { "dsh:yes" } else { "dsh:no " };
                println!("{:<26} {oc} {ds} {}", s.name, s.description);
            }
            ExitCode::SUCCESS
        }
        "--emit-opencode" => {
            let path = agents_root().join("generated/opencode/mcp.servers.json");
            match write_if_changed(&path, &render_opencode_block(&servers)) {
                Ok(c) => {
                    println!("{}: {}", if c { "更新" } else { "変更なし" }, path.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("mcp-sync: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "--apply" => {
            let jsonc = home().join(".config/opencode/opencode.jsonc");
            match apply_opencode(&servers, &jsonc) {
                Ok(true) => {
                    println!("更新 {}", jsonc.display());
                    ExitCode::SUCCESS
                }
                Ok(false) => {
                    println!("変更なし {}", jsonc.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("mcp-sync: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "--emit-dsh" => {
            let path = agents_root().join("generated/dsh/mcp-servers.patch.yml");
            match write_if_changed(&path, &emit_dsh(&servers)) {
                Ok(c) => {
                    println!("{}: {}", if c { "更新" } else { "変更なし" }, path.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("mcp-sync: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "--check-dsh" => {
            let profiles: Vec<(&str, PathBuf)> = vec![
                ("web", home().join(".dsh/profiles/web/cordis.patch.yml")),
                (
                    "agentrouter",
                    home().join(".dsh-agentrouter/profiles/agentrouter/cordis.patch.yml"),
                ),
            ];
            match check_dsh(&servers, &profiles) {
                Ok(0) => {
                    println!("mcp-sync: DSH プロファイルは正本と一致（{} サーバ）", servers.len());
                    ExitCode::SUCCESS
                }
                Ok(n) => {
                    eprintln!("mcp-sync: DSH に {n} 件の差分があります");
                    ExitCode::FAILURE
                }
                Err(e) => {
                    eprintln!("mcp-sync: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            let jsonc = home().join(".config/opencode/opencode.jsonc");
            match check_opencode(&servers, &jsonc) {
                Ok(0) => {
                    println!("mcp-sync: opencode.jsonc は正本と一致（{} サーバ）", servers.len());
                    ExitCode::SUCCESS
                }
                Ok(n) => {
                    eprintln!("mcp-sync: opencode.jsonc に {n} 件の差分があります");
                    ExitCode::FAILURE
                }
                Err(e) => {
                    eprintln!("mcp-sync: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
