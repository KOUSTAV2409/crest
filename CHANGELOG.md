# Changelog

All notable changes to this project are documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/).
 
## [0.2.7] - 2026-05-09

### Added
- **Base64 IPC Streaming**: upgraded the Tauri IPC bridge to use Base64 encoding for search results, eliminating massive JSON array serialization overhead and ensuring buttery-smooth 60+ FPS UI rendering with zero input lag.
- **Extreme Binary Optimizations**: tuned `Cargo.toml` with `lto = "fat"`, `codegen-units = 1`, and `opt-level = "s"` to drastically reduce the size of the final compiled binaries (< 10MB) and accelerate cold start times.
- **Linux CSD Workaround**: implemented a dynamic post-realization window flag update to force GNOME/Wayland compositors to drop the thick client-side decoration titlebars, ensuring a seamless, transparent palette experience.

## [0.2.6] - 2026-05-06

### Added
- **AppStream Metadata**: integrated `metainfo.xml` for professional presentation in Linux software stores (GNOME Software, Discover).
- **DEB Bundling**: configured Tauri to officially package the metadata file into `/usr/share/metainfo/`.

### Changed
- **Branding**: refined primary colors for high-fidelity carousel tiles in app stores.
 
## [0.2.5] - 2026-05-06

### Changed
- **Stitch High-Fidelity UI**: migrated the entire launcher aesthetic to match the premium "Web Page Replicator" design from Stitch.
- **50/50 Split Layout**: redesigned the command palette container for a balanced 50/50 split between results and the preview panel.
- **Glassy Command Palette**: updated background colors (`#12121e`), borders (`#2d2d3d`), and accents (`#7c4dff`) for a more professional, "command center" feel.
- **Refined Preview Panel**: implemented a left-aligned hero section, glassy icon wrappers, and a HUD-style shortcut tip box.
- **Minimalist Search Header**: simplified the search field with borderless inputs and larger typography for better readability.

### Added
- **Performance instrumentation**: high-resolution timing logs (`[STARTUP]`) added to the Rust backend to monitor bottleneck regressions.

### Changed
- **Async Startup (Zero-Block)**: moved heavy indexers (Apps, Files, Clipboard) and Hotkey registration to background threads. **Main thread setup time reduced from ~2s to 1.5ms (1000x improvement)**.
- **Atomic App Indexing**: app scans now perform a "Scan-then-Swap" atomic transaction, preventing blank search results during background refreshes.
- **Database Concurrency**: enabled **WAL mode** (Write-Ahead Logging) for the indexer database to allow parallel background writes without locking the search UI.


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
[0.2.4]: https://github.com/KOUSTAV2409/crest/releases/tag/v0.2.4
[0.2.5]: https://github.com/KOUSTAV2409/crest/releases/tag/v0.2.5
[0.2.6]: https://github.com/KOUSTAV2409/crest/releases/tag/v0.2.6
[0.2.7]: https://github.com/KOUSTAV2409/crest/releases/tag/v0.2.7
