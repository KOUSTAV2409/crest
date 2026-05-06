# Crest v0.2.4 — The Performance Update

This release focuses on radical startup performance and background data stability.

### 🏔️ Highlights:
- **1,000x Faster Startup**: The main window now appears in ~1.5ms. We moved heavy system scans (Apps, Files, Clipboard) to background threads so they never block your flow.
- **Atomic App Indexing**: Refactored the app indexer to use a "Scan-then-Swap" strategy. Results are always populated, even during a background refresh.
- **High-Concurrency Database**: Enabled **WAL mode** (Write-Ahead Logging) for the internal SQLite database, allowing parallel background indexing without locking the UI.
- **Performance Diagnostics**: Added high-resolution timing logs (`[STARTUP]`) to the Rust backend to ensure zero regressions in speed.

### 🛠️ Technical Changes:
- Moved `hotkey::init`, `indexer::apps::init`, `indexer::files::init`, and `clipboard::init` into background tasks.
- Implemented `Sqlite` atomic transactions for the FTS5 app index.
- Optimized Freedesktop `.desktop` file parsing to skip `NoDisplay=true` entries.
