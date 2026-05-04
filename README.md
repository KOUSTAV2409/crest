# <p align="center">🏔️ Crest</p>
<p align="center"><b>The Raycast experience, finally on Linux.</b></p>

<p align="center">
  <img src="website/assets/hero-mockup.png" alt="Crest Preview" width="800">
</p>

<p align="center">
  <a href="https://github.com/KOUSTAV2409/crest/releases">
    <img src="https://img.shields.io/github/v/release/KOUSTAV2409/crest?style=flat-square&color=c2c1ff" alt="Latest Release">
  </a>
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square&color=4d49d7" alt="License">
  <img src="https://img.shields.io/badge/built_with-Rust-orange?style=flat-square&logo=rust" alt="Built with Rust">
</p>

---

**Crest** is a high-performance, keyboard-first productivity launcher for the Linux desktop. Built with **Rust** and **Tauri**, it delivers a sub-50ms execution speed and a premium design language inspired by Raycast.

## ✨ Key Features

- 📋 **Clipboard History**: Native Rust listener with secure local persistence. Search back through days of copies instantly.
- 🌐 **Instant Web Search**: Privacy-respecting DuckDuckGo bridge with instant answers (Wikipedia, Calculator, etc.).
- 🚀 **App Launcher**: Blazing fast application indexing with fuzzy search and shortcut support.
- 🧩 **Plugin Engine**: Extensions run as separate processes; by default only scripts listed in `~/.config/crest/plugins/manifest.json` are loaded (see `configs/plugins.manifest.example.json`).
- 💎 **Premium UI**: Ultra-sharp glassmorphism, ⌘K action bars, and smooth Framer Motion transitions.
- 🔒 **Privacy First**: 100% local-first. No cloud syncing, no telemetry, no accounts.

## 🛠️ Technical Specs

- **Core**: Pure Rust for system integration (Clipboard, Indexing, Search).
- **UI**: React + TypeScript + Vite, with component-scoped CSS and shared variables in `index.css` (no Tailwind in this repo).
- **Database**: Local SQLite for high-speed metadata and history storage.
- **Footprint**: < 40MB RAM idle usage.

## 🚀 Installation & Setup

### 1. Download
Head to [GitHub Releases](https://github.com/KOUSTAV2409/crest/releases) and download the latest asset for your distribution:
- **Debian/Ubuntu**: `.deb` package
- **Generic Linux**: `.AppImage`

### 2. Install
- **Debian/Ubuntu**: `sudo apt install ./crest_0.2.0_amd64.deb`
- **Generic Linux**: `chmod +x crest.AppImage && ./crest.AppImage`

### 3. Global shortcut
Crest registers a **global hotkey** from your config (default **`super+Space`**, i.e. Windows/Meta + Space). Edit `~/.config/crest/config.json` and set `global_shortcut` to any string supported by [global-hotkey](https://docs.rs/global-hotkey) (modifiers first), e.g. `alt+Space`, `super+K`, `control+shift+KeyP`. Alternatively, you can still launch the binary from a DE shortcut bound to `crest`.

### 4. Plugins (extensions)
By default **`plugin_policy` is `"manifest"`**: only entries in `~/.config/crest/plugins/manifest.json` run. Copy `configs/plugins.manifest.example.json` into that path, list your scripts with relative paths, and `chmod +x` them as needed. To opt back into the legacy “any file in the folder is runnable” model (full trust), set `"plugin_policy": "open"` in `config.json`.

## ⌨️ How to Use

- **Search**: Start typing to find apps, files, or calculate math.
- **Navigate**: Use `Up` and `Down` arrows to select results.
- **Execute**: Press `Enter` to launch the primary action.
- **Actions**: Press `Cmd + K` (or `Ctrl + K`) to open the secondary action menu.
- **Modes**: Use `Backspace` on an empty search bar to switch between Clipboard, Search, and File modes.

## 🛠️ Development Setup (For Contributors)

Install Rust, Node 18+, and dependencies (`npm install`). Use **`npm run tauri:dev`** so the native shell is built with the optional **`devtools`** Cargo feature (WebView inspector). Release builds and CI use **`cargo build --release` / `tauri build` without that flag.

## 🗺️ Roadmap
- [ ] **Deep AI Integration**: Local LLM support for context-aware commands.
- [ ] **Plugin Marketplace**: A central registry for community extensions.
- [ ] **Mobile Remote**: Control your Linux desktop from your phone via local network.

## 📄 License
MIT © [Koustav](https://github.com/KOUSTAV2409)
