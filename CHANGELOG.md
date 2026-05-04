# Changelog

All notable changes to this project are documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/).

## [0.2.0] - 2026-05-04

### Added
- **`~/.config/crest/config.json`**: configurable global hotkey (`global_shortcut`, default `super+Space`) and `plugin_policy`.
- **Plugin manifest**: default policy `manifest` loads only entries from `~/.config/crest/plugins/manifest.json`; example at `configs/plugins.manifest.example.json`.
- **Incremental file index**: tracks mtime/size with a generation sweep; **fs watcher** (debounced) re-syncs Documents / Downloads / Desktop.
- **SQL prefilter** for app/file search before fuzzy matching (with bounded fallback scans).
- **`src/types/ipc.ts`**: typed IPC shapes for search results on the frontend.
- **CSP** for the webview in `tauri.conf.json` (tighter than `null`).
- **`npm run tauri:dev`** enables the optional **`devtools`** Cargo feature for local debugging.

### Changed
- **Browser search** from the palette opens **DuckDuckGo**; UI copy aligned (README, DOCUMENTBOOK, in-app labels).
- **Plugin execution**: invokes script/binary with argv (no `sh -c` concatenation); **`plugin_policy: open`** restores legacy “any file in plugins dir” behavior (full trust).
- **Release vs dev**: `devtools` is **not** enabled on `crest` by default; use `npm run tauri:dev`.

### Migration
- If you used extensions as **loose files** in `~/.config/crest/plugins/`, add **`manifest.json`** or set **`plugin_policy` to `"open"`** in `config.json`.

[0.2.0]: https://github.com/KOUSTAV2409/crest/releases/tag/v0.2.0
