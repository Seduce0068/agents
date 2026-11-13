//! ```cargo
//! [dependencies]
//! serde_json = "1"
//! ```
//!
//! サブエージェント定義の同期ツール。
//!
//! 正本:  `~/.agents/agents/<name>.md`（frontmatter: name / description / access、本文=システムプロンプト）
//! 上書き: `~/.agents/harness/<harness>/agents.json`（ハーネス固有の model / 権限表など）
//!
//! 使い方:
//!   agents-sync.rs --check          生成結果と実ファイルの差分を検査（差分があれば exit 1）
//!   agents-sync.rs --apply          opencode の agents/*.md を生成して書き込む
//!   agents-sync.rs --emit-opencode  生成結果を generated/opencode/agents/ へ出力
//!   agents-sync.rs --emit-dsh        DSH の agent-preset パッチを generated/dsh/ へ出力
//!   agents-sync.rs --list           正本の一覧を表示

use std::collections::BTreeMap;
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

fn opencode_agents_dir() -> PathBuf {
    match env::var("OPENCODE_CONFIG_DIR") {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v).join("agents"),
        _ => home().join(".config/opencode/agents"),
    }
}

/// 1 体の中立定義。
#[derive(Debug, Clone)]
struct Agent {
    name: String,
    description: String,
    access: String,
    body: String,
}

/// frontmatter と本文を分離する。frontmatter は `---` で囲まれた単純な key: value。
fn split_frontmatter(text: &str) -> Option<(BTreeMap<String, String>, String)> {
    let rest = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n"))?;
    let end = rest.find("\n---")?;
    let front = &rest[..end];
    let after = &rest[end + 4..];
    let after = after.strip_prefix('\n').unwrap_or(after);
    let mut map = BTreeMap::new();
    for line in front.lines() {
        let Some((k, v)) = line.split_once(':') else { continue };
        let key = k.trim();
        if key.is_empty() || key.starts_with('#') {
            continue;
        }
        map.insert(key.to_string(), v.trim().to_string());
    }
    Some((map, after.to_string()))
}

fn load_agents() -> Result<Vec<Agent>, String> {
    let dir = agents_root().join("agents");
    let mut out = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (fm, body) = split_frontmatter(&text)
            .ok_or_else(|| format!("{}: frontmatter がありません", path.display()))?;
        let name = fm
            .get("name")
            .cloned()
            .ok_or_else(|| format!("{}: name がありません", path.display()))?;
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        if stem != name {
            return Err(format!("{}: ファイル名と name が不一致 ({stem} != {name})", path.display()));
        }
        out.push(Agent {
            name,
            description: fm.get("description").cloned().unwrap_or_default(),
            access: fm.get("access").cloned().unwrap_or_else(|| "read-only".into()),
            body: body.trim_start_matches('\n').trim_end().to_string(),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// opencode 用の上書き定義。
#[derive(Debug, Clone, Default)]
struct OpencodeOverride {
    model: String,
    fallbacks: Vec<String>,
}

fn load_opencode_overrides() -> Result<(BTreeMap<String, OpencodeOverride>, serde_json::Value), String> {
    let path = agents_root().join("harness/opencode/agents.json");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut map = BTreeMap::new();
    if let Some(agents) = doc.get("agents").and_then(|v| v.as_object()) {
        for (name, cfg) in agents {
            let model = cfg.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let fallbacks = cfg
                .get("fallbackModels")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            map.insert(name.clone(), OpencodeOverride { model, fallbacks });
        }
    }
    Ok((map, doc))
}

/// 権限表を opencode の YAML 断片へ変換する。
fn permission_yaml(doc: &serde_json::Value, access: &str) -> Result<String, String> {
    let rules = doc
        .get("accessPermissions")
        .and_then(|v| v.get(access))
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("accessPermissions.{access} がありません"))?;
    let mut out = String::from("permissions:\n");
    for rule in rules {
        let action = rule.get("action").and_then(|v| v.as_str()).unwrap_or("*");
        let resource = rule.get("resource").and_then(|v| v.as_str()).unwrap_or("*");
        let effect = rule.get("effect").and_then(|v| v.as_str()).unwrap_or("ask");
        out.push_str(&format!(
            "  - action: {action}\n    resource: \"{resource}\"\n    effect: {effect}\n"
        ));
    }
    Ok(out)
}

/// opencode のエージェント Markdown を生成する。
fn render_opencode(agent: &Agent, ov: &OpencodeOverride, doc: &serde_json::Value) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("description: {}\n", agent.description));
    out.push_str("mode: subagent\n");
    out.push_str("# この file は ~/.agents/agents から生成される。直接編集しない。\n");
    out.push_str("# 正本: ~/.agents/agents/");
    out.push_str(&agent.name);
    out.push_str(".md / 上書き: ~/.agents/harness/opencode/agents.json\n");
    out.push_str(&format!("model: {}\n", ov.model));
    for fb in &ov.fallbacks {
        out.push_str(&format!("# model: {fb}\n"));
    }
    out.push_str(&permission_yaml(doc, &agent.access)?);
    out.push_str("---\n\n");
    out.push_str(&agent.body);
    out.push('\n');
    Ok(out)
}

/// DSH の agent-preset パッチ（read-only レビュアー等）を生成する。
fn render_dsh_presets(agents: &[Agent]) -> String {
    let mut out = String::new();
    out.push_str(
        "# このパッチは ~/.agents/bin/agents-sync.rs が生成する。直接編集しない。\n\
         # 適用方法（検証してから）:\n\
         #   DSH_HOME=~/.dsh dsh --profile agentrouter \\\n\
         #     --patch ~/.agents/generated/dsh/agent-presets.patch.yml --dump-config\n\
         # 注意: model による選択には各 preset 行の tool-subagent に\n\
         #       allowedAgentPresets を追加する必要がある。\n\
         #\n\
         # 生成元: ~/.agents/agents/*.md\n\n",
    );
    out.push_str("- insert:\n");
    for (i, agent) in agents.iter().enumerate() {
        let read_only = agent.access == "read-only" || agent.access == "tests-only";
        let write = if read_only { "false" } else { "true" };
        let edit = write;
        out.push_str(&format!("    - id: preset-{}\n", agent.name));
        out.push_str("      name: '@deepseek-ai/dsh-agent-preset'\n");
        out.push_str("      config:\n");
        out.push_str(&format!("        id: {}\n", agent.name));
        out.push_str(&format!("        name: {}\n", agent.name));
        out.push_str(&format!(
            "        description: '{}'\n",
            agent.description.replace('\'', "''")
        ));
        out.push_str(&format!("        order: {}\n", 10 + i));
        out.push_str("        plugins:\n");
        out.push_str("          - id: persona\n");
        out.push_str("            name: '@deepseek-ai/dsh-persona'\n");
        out.push_str("            config:\n");
        out.push_str("              prefix: |-\n");
        for line in agent.body.lines() {
            if line.is_empty() {
                out.push('\n');
            } else {
                out.push_str(&format!("                {line}\n"));
            }
        }
        // complete: true はシステムプロンプト全体（ツール案内を含む）を置換するため使わない。
        // 役割は persona prefix として先頭に付与し、標準のツール案内と実行時コンテキストは維持する。
        out.push_str("          - id: agent-instructions\n");
        out.push_str("            name: '@deepseek-ai/dsh-agent-instructions'\n");
        out.push_str("            config:\n");
        out.push_str("              maxBytes: 65536\n");
        out.push_str("          - id: time-context\n");
        out.push_str("            name: '@deepseek-ai/dsh-time-context'\n");
        out.push_str("          - id: tool-fs\n");
        out.push_str("            name: '@deepseek-ai/dsh-tool-fs'\n");
        out.push_str("            config:\n");
        out.push_str("              applyPatch: false\n");
        out.push_str(&format!("              write: {write}\n"));
        out.push_str(&format!("              edit: {edit}\n"));
        out.push_str("          - id: tool-fs-search\n");
        out.push_str("            name: '@deepseek-ai/dsh-tool-fs-search'\n");
        out.push_str("          - id: skill-filesystem\n");
        out.push_str("            name: '@deepseek-ai/dsh-skill-filesystem'\n");
        out.push_str("          - id: tool-skill\n");
        out.push_str("            name: '@deepseek-ai/dsh-tool-skill'\n");
        out.push_str("          - id: tool-web\n");
        out.push_str("            name: '@deepseek-ai/dsh-tool-web'\n");
        out.push_str("            config:\n");
        out.push_str("              fetch: true\n");
        out.push_str("              searchTimeoutMs: 60000\n");
    }
    out.push_str(HEADER_SELECTION_GUIDANCE);
    for a in agents {
        out.push_str(&format!("#       - {}\n", a.name));
    }
    out.push_str("#       - standard\n");
    out
}

/// プリセット選択を有効化する場合の手順（生成物にコメントとして付与する）。
const HEADER_SELECTION_GUIDANCE: &str = concat!(
    "\n# --- 選択を有効にする場合（任意・上級者向け） ---\n",
    "# Agent Teams を有効にしたまま使う場合、spawn_teammate はプリセットを選べない。\n",
    "# subagent ツールでプリセット選択を使うには、agent-team 系を無効化して\n",
    "# tool-subagent を有効化し allowedAgentPresets を渡す（下記のコメントを外す）。\n",
    "# 注意: これは Agent Teams を置き換える変更。現在の運用に合わせて判断すること。\n",
    "#\n",
    "# - id: tool-agent-team\n",
    "#   disabled: true\n",
    "# - id: agent-team\n",
    "#   disabled: true\n",
    "# - id: ui-agent-team\n",
    "#   disabled: true\n",
    "# - id: tool-subagent\n",
    "#   config:\n",
    "#     provider: spawn\n",
    "#     toolName: subagent\n",
    "#     modelSelection: true\n",
    "#     modelDiscovery: true\n",
    "#     allowedAgentPresets:\n",
);

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

    let agents = match load_agents() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("agents-sync: 正本の読み込みに失敗: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (overrides, doc) = match load_opencode_overrides() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("agents-sync: 上書き定義の読み込みに失敗: {e}");
            return ExitCode::FAILURE;
        }
    };

    if mode == "--list" {
        for a in &agents {
            println!("{:<20} access={:<12} {}", a.name, a.access, a.description);
        }
        return ExitCode::SUCCESS;
    }

    if mode == "--check-dsh" {
        // ホーム直下 $DSH_HOME/cordis.patch.yml が生成物と一致するかを検査する。
        let expected = render_dsh_presets(&agents);
        let mut drift = false;
        for home_env in ["DSH_HOME", "DSH_AGENTROUTER_HOME"] {
            let _ = home_env;
        }
        let homes = [home().join(".dsh"), home().join(".dsh-agentrouter")];
        for h in &homes {
            let path = h.join("cordis.patch.yml");
            match fs::read_to_string(&path) {
                Ok(existing) if existing == expected => {
                    println!("OK   {}", path.display());
                }
                Ok(_) => {
                    println!("drift: {} (生成物と不一致)", path.display());
                    drift = true;
                }
                Err(_) => {
                    println!("未適用: {} （--emit-dsh の出力を配置してください）", path.display());
                    drift = true;
                }
            }
        }
        if drift {
            eprintln!("agents-sync: DSH ホームのパッチが未適用または不一致です。");
            return ExitCode::FAILURE;
        }
        println!("agents-sync: DSH ホームのプリセットは同期済み");
        return ExitCode::SUCCESS;
    }

    if mode == "--emit-dsh" {
        let path = agents_root().join("generated/dsh/agent-presets.patch.yml");
        match write_if_changed(&path, &render_dsh_presets(&agents)) {
            Ok(changed) => {
                println!("{}: {}", if changed { "更新" } else { "変更なし" }, path.display());
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("agents-sync: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    // opencode 出力を組み立てる
    let mut rendered: BTreeMap<String, String> = BTreeMap::new();
    for agent in &agents {
        let Some(ov) = overrides.get(&agent.name) else {
            eprintln!("agents-sync: {} の上書き定義がありません", agent.name);
            return ExitCode::FAILURE;
        };
        match render_opencode(agent, ov, &doc) {
            Ok(text) => {
                rendered.insert(agent.name.clone(), text);
            }
            Err(e) => {
                eprintln!("agents-sync: {}: {e}", agent.name);
                return ExitCode::FAILURE;
            }
        }
    }

    let mut drift = false;
    match mode {
        "--emit-opencode" => {
            for (name, text) in &rendered {
                let path = agents_root().join(format!("generated/opencode/agents/{name}.md"));
                if let Err(e) = write_if_changed(&path, text) {
                    eprintln!("agents-sync: {e}");
                    return ExitCode::FAILURE;
                }
            }
            println!("generated/opencode/agents/: {} ファイル", rendered.len());
        }
        "--apply" => {
            for (name, text) in &rendered {
                let path = opencode_agents_dir().join(format!("{name}.md"));
                match write_if_changed(&path, text) {
                    Ok(true) => println!("更新 {}", path.display()),
                    Ok(false) => println!("変更なし {}", path.display()),
                    Err(e) => {
                        eprintln!("agents-sync: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
        }
        "--check" | _ => {
            for (name, text) in &rendered {
                let path = opencode_agents_dir().join(format!("{name}.md"));
                match fs::read_to_string(&path) {
                    Ok(existing) if existing == *text => {}
                    Ok(_) => {
                        println!("drift: {} (実ファイルと生成結果が不一致)", path.display());
                        drift = true;
                    }
                    Err(e) => {
                        println!("drift: {} (読み込み不可: {e})", path.display());
                        drift = true;
                    }
                }
            }
            // 実ディレクトリに余分なファイルがないか
            if let Ok(entries) = fs::read_dir(opencode_agents_dir()) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("md") {
                        continue;
                    }
                    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
                    if !rendered.contains_key(stem) {
                        println!("drift: {} (正本に対応する定義がない)", path.display());
                        drift = true;
                    }
                }
            }
            if drift {
                eprintln!("agents-sync: 差分があります。--apply で同期してください。");
                return ExitCode::FAILURE;
            }
            println!("agents-sync: 同期済み（{} 体）", rendered.len());
        }
    }

    if drift {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
