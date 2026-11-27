---
type: Reference
title: "環境・GUI アプリ・シェルの詳細手順"
description: "sudo/kiri、Flatpak user スコープ、クロスプラットフォーム、GUI 検証とメニュー配置"
tags:
  - environment
  - gui
  - kubuntu
---

## ユーザー環境

- PC: Kubuntu（Linux）をメイン開発環境とし、実機動作検証にも対応する。
- スマートフォン: Android（Edge / Violentmonkey）と iOS（PWA）に対応する。
- 回線・キャリア等の固有情報は記載しない。
- 開発ツール: [mise](https://mise.jdx.dev/) を想定。
- Windows向け手順ではなく、Linux/Kubuntu向けの手順を提供する。

## 権限昇格とコンテナランタイム

- `sudo` はパスキー／顔認証による**対話承認**で利用できる。`sudo -n`（非対話）が失敗しても「sudo は使えない」と断定しない。権限が必要な操作ではユーザーに承認を促す形で `sudo` を実行する。
- コンテナ操作はまず `kiri`（`/usr/bin/kiri`、Rust 製の Docker 互換 CLI）を確認する。Docker CLI も導入済みだが、セッションによって `docker` グループが反映されず `permission denied` になることがある。その場合は再ログイン、`newgrp docker`、または上記の対話 `sudo` を使う。
- `kiri daemon --help` は終了せずハングすることがあるため、ヘルプ確認では `timeout` を付ける。
- 2026-11-22 時点で `kubectl` / `kind` / `k3d` は `~/.local/bin` に導入済み。バージョンは変わり得るため、必要なら `kubectl version --client`、`kind version`、`k3d version` で確認する。

## Flatpak のユーザースコープ

- `flatpak install --user` では依存解決にユーザースコープのリモートだけが使われる。system スコープに `flathub` があっても参照されず、`error: ... requires the runtime org.kde.Platform/x86_64/6.10 which was not found` のように失敗する。
- 対処: 先に `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo` を実行し、アプリ固有のリモートも `--user` で追加する。
- 実例: 2026-11-24 の Ladybird α（2026.47.7）インストールで再現し、上記で解消した（root 権限不要で導入可能）。

## クロスプラットフォーム対応方針

- **Kubuntu（Linux）**: メイン開発環境とし、実機動作検証にも対応する。
- **Android**: アプリ版とPWAの両方に対応する。
- **iOS**: アプリ版は提供せず、PWAに対応する。
- **Windows**: 評価版Windowsで動作検証する。
- **macOS**: 実機動作検証は行わない。

## ブラウザ対応対象

- Webフロントエンド、ユーザースクリプト、拡張機能等でブラウザ対応を検討・実装・検証する場合、対象ブラウザは Chrome・Edge・Brave・Firefox・Safari を基本とする。

## シェルとコマンド提示

- 対話シェルとスクリプトは bash を使う。ユーザーに提示するコマンド例は bash で動作する構文にする（Nushell/zsh/fish固有の構文を提示しない）。
- スクリプトは `#!/usr/bin/env bash` と `set -euo pipefail` を基本とする。
- `$(...)`、`&&`、`if ...; then ...; fi`、`for ...; do ...; done`、ヒアドキュメント（`<<'EOF'`）を使う。
- 環境変数は `export VAR=value`、終了コードは `$?` で扱う。配列は `arr=(a b c)`、連想配列は `declare -A`。
- 外部コマンドに `^` を付けない。パイプとリダイレクトはPOSIX準拠で書く。
- 提示前に `bash -n`、`shellcheck`、`shfmt -d` で検証する。構造化データは `jq`/`yq`/`jc` で扱い、シェル内でJSONを手組みしない。
- ユーザーに実行してもらう確認コマンドは1つのコードブロックにまとめ、コピー&ペーストで順に実行できる形にする。各コマンドの前に `echo` で見出しを付け、実行環境（Kubuntu / リモートホスト等）を直前に明記する。確認と修正が混在する場合は「確認 → 修正 → 再確認」の順に同じブロック内へ並べる。

## 実行環境での自己確認

- エージェントが実行できる確認は、ユーザーに依頼せず自分で行う。対象はコマンドの実行結果、終了コード、ファイルやディレクトリの存在、バージョン、設定値、環境変数、プロセスの有無、ネットワーク到達性など。
- ユーザーへ依頼するのは、エージェントから実行できない操作に限る。物理的な操作、画面やUIの目視確認、権限昇格が必要な操作、アクセス権限を持たないホストや実機での操作、対話的な認証入力、秘密情報の入力。
- 実行が拒否された場合や環境制約で確認できない場合のみ、理由と確認手順を示してユーザーへ依頼する。結果を推測で補わない。
- 確認結果は Runtime Proof として扱い、過去のログや記憶より優先する。
- GUIアプリの動作確認は、ユーザーに委ねずエージェント自身で検証する。Xwayland(X11)アプリは `xwininfo -root -tree` / `xprop`、Waylandネイティブは `kdotool` 等で確認する。取得できない場合は理由を添えてユーザーに確認を依頼する。

## GUIアプリのビルド成果物をアプリケーションメニューへ配置

- `cargo build --release` だけではバイナリが `target/release/` に生成されるだけで、KDE のアプリケーションメニューには登録されない。ビルド後に成果物を使える状態にするには、リポジトリが用意しているローカルインストールスクリプトを実行する。
- GPUI Task Manager の場合は `bash scripts/install-local.sh` を使う。これは root 不要で、`cargo build --release` → `~/.local/bin/gpui-task-manager` → `~/.local/share/applications/gpui-task-manager.desktop` → `~/.local/share/icons/hicolor/` へのアイコン配置 → `update-desktop-database` / `kbuildsycoca6` によるメニューキャッシュ更新を一括で行う。再ビルド後も同じコマンドを再実行すれば、バイナリとメニューが最新化される。既存バイナリを再利用する場合は `--skip-build` を付ける。
- 配布用 Debian パッケージは `packaging/linux/build-deb.sh` が作成する。こちらは `/usr/bin` と `/usr/share/applications` 配下へインストールされるため、開発中の反復には `scripts/install-local.sh`、配布には `.deb` を使い分ける。
- エージェントがビルド成果物を「使える状態にした」と報告する前に、ランチャーエントリが実際に存在すること（`desktop-file-validate` と `kbuildsycoca6` 後のメニュー確認）を Runtime Proof として確認する。

## Edge/Chromium 拡張機能のPWA化（タスクバー独立表示）

- 拡張機能（`extension://` や `chrome-extension://`）を `--app` で起動する際、KDE Plasma等のタスクバーでEdge本体とグループ化されるのを防ぐには、実際のウィンドウクラス（`WM_CLASS`）に `.desktop` の `StartupWMClass` を一致させる。
- 拡張機能のクラス名は単なるIDではなく `[拡張機能ID]__index.html`（アンダースコア2つでメインファイルを連結）などになるため、実際のウィンドウ情報を調査してから `StartupWMClass` を指定する。
