# hooks/ (参考設定)

`hooks.json` は Codex 系フックの参考設定です。
`PostToolUse` が参照する `scripts/track-telemetry.sh` / `.ps1` は
このリポジトリには含まれていません（ローカル運用時に別途配置する）。

誤って有効化して実行失敗しないよう、利用時は参照先スクリプトの存在を
確認してから `hooks.json` を設定してください。
