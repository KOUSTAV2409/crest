# Changelog

All notable changes to this project are documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/).

## [0.2.3] - 2026-05-05

### Added
- **Freedesktop icon resolution** (theme-aware) with non-blocking IPC and UI-side idle scheduling so icons don’t stall typing.
- **Web results fallback**: when DuckDuckGo returns a bot challenge page, Crest augments results using Wikipedia’s search API.

### Fixed
- **Input lag on first character**: heavy search/icon work no longer blocks the UI event loop; keystrokes render immediately.

### Changed
- **UI polish**: search field, result shortcuts, and preview panel updated to match the hero mockup styling.

## [0.2.2] - 2026-05-05

### Added
- **In-app shortcut help on Linux**: amber banner explains that **Wayland** often blocks in-app Win/Super shortcuts; directs you to bind **`crest`** in system keyboard settings or use Xorg / another key combo.
- **`get_shortcut_setup_hint` IPC** and tracking of successful global hotkey registration.

## [0.2.1] - 2026-05-04

### Fixed
- **Wayland-friendly launcher toggle**: integrated [`tauri-plugin-single-instance`](https://v2.tauri.app/plugin/single-instance/). Running **`crest`** while the app is already open **shows or hides** the main window — so desktop shortcuts (“Custom shortcut” → `crest`) behave like Raycast/Crest on desktops where internal **Super+Space** grabs fail.

### Changed
- **global-hotkey** updated to **0.8** (x11rb-based path on Linux).
- **Diagnostics**: shortcut parse/register failures append **`~/.local/share/crest/hotkey.log`** (and stderr) with concrete guidance.

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
[0.2.1]: https://github.com/KOUSTAV2409/crest/releases/tag/v0.2.1
[0.2.2]: https://github.com/KOUSTAV2409/crest/releases/tag/v0.2.2
[0.2.3]: https://github.com/KOUSTAV2409/crest/releases/tag/v0.2.3
