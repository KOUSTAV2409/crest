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

**Crest** is a high-performance, keyboard-first productivity launcher for the Linux desktop — a Raycast-style launcher built with **Rust** + **Tauri**.

It started as a **personal tool** (I felt the gap on Linux and built the launcher I wanted). It’s now packaged so others can use it too — with a focus on **performance**, **privacy**, and a **premium UI**.

### Trust & safety
- **Local-first**: no accounts, no telemetry, no cloud sync.
- **Plugins are manifest-scoped by default**: only explicitly listed scripts run.
- **Transparent**: MIT licensed and built in the open.

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

## 🧠 How it works (high level)

- **App search**: `.desktop` entries are indexed into a local SQLite database, then filtered + ranked with a fuzzy matcher for fast “type to launch” workflows.
- **Freedesktop icons**: app icons are resolved via the system’s icon theme (e.g. Adwaita/Yaru/Papirus/hicolor) and cached; the UI loads them lazily to keep typing smooth.
- **Wayland hotkeys**: on some Wayland desktops, global “Super/Meta” grabs are restricted. Crest supports a single-instance toggle so a system shortcut can simply run `crest` to show/hide the palette.
- **Clipboard history**: captured locally, searchable instantly; nothing leaves your machine.
- **Plugins**: extensions run as separate processes. By default, Crest only runs scripts listed in `~/.config/crest/plugins/manifest.json` (manifest-scoped for safety).
- **Web answers**: uses DuckDuckGo instant answers where available and falls back when SERP HTML is blocked (bot checks can happen).

## 🚀 Installation & Setup

Hosted user documentation mirrors this README and `DOCUMENTBOOK.md`: open [`website/documentation.html`](website/documentation.html) in the deployed site bundle (same path live on crest.run-style hosts).

### 1. Download
Head to [GitHub Releases](https://github.com/KOUSTAV2409/crest/releases) and download the latest asset for your distribution:
- **Debian/Ubuntu**: `.deb` package
- **Generic Linux**: `.AppImage`

### 2. Install
- **Debian/Ubuntu**: `sudo apt install ./crest_0.2.3_amd64.deb`
- **Generic Linux**: `chmod +x crest.AppImage && ./crest.AppImage`

### 3. Global shortcut
Crest registers a **global hotkey** from your config (default **`super+Space`**, i.e. Windows/Meta + Space). Edit `~/.config/crest/config.json` and set `global_shortcut` to any string supported by [global-hotkey](https://docs.rs/global-hotkey) (modifiers first), e.g. `alt+Space`, `super+K`, `control+shift+KeyP`.

**If Super+Space does nothing:** you may be on **Wayland**. Many desktops block the X11-style “global grab” Crest uses internally; failures are easy to miss (they’re not shown in the UI). Check **`~/.local/share/crest/hotkey.log`** (also printed to stderr if you launch `crest` from a terminal).

**Recommended fix:** open **Settings → Keyboard → Custom Shortcuts**, add **Super+Space** (or another key), and set the command to **`crest`**. Crest is a **single instance** (**v0.2.1+**): each extra run of **`crest`** **shows or hides** the palette — the reliable approach on Wayland. **v0.2.2+** also shows an in-app banner explaining this.

#### 🐧 Advanced: Setup via Terminal (GNOME/Ubuntu)
If you prefer the command line or are automating your setup, you can register the shortcut directly into the GNOME dconf database:

1. **Register the path**:
   ```bash
   gsettings set org.gnome.settings-daemon.plugins.media-keys \
     custom-keybindings "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/']"
   ```

2. **Configure the shortcut**:
   ```bash
   # Set the Name
   gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ \
     name "Crest"

   # Set the Command (ensure 'crest' is in your /usr/bin or use full path)
   gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ \
     command "crest"

   # Set the Binding (<Super>space for Windows+Space)
   gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ \
     binding "<Super>space"
   ```

*Note: Replace `custom0` with `custom1`, `custom2`, etc., if you already have other custom shortcuts.*

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
