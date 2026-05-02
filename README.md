# <p align="center">🏔️ Crest</p>
<p align="center"><b>The Raycast experience, finally on Linux.</b></p>

<p align="center">
  <img src="website/assets/hero-mockup.png" alt="Crest Preview" width="800">
</p>

<p align="center">
  <a href="https://github.com/koustav/crest/releases">
    <img src="https://img.shields.io/github/v/release/koustav/crest?style=flat-square&color=c2c1ff" alt="Latest Release">
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
- 🧩 **Plugin Engine**: Extensible architecture supporting JS, Python, and Bash scripts.
- 💎 **Premium UI**: Ultra-sharp glassmorphism, ⌘K action bars, and smooth Framer Motion transitions.
- 🔒 **Privacy First**: 100% local-first. No cloud syncing, no telemetry, no accounts.

## 🛠️ Technical Specs

- **Core**: Pure Rust for system integration (Clipboard, Indexing, Search).
- **UI**: React + TypeScript + Tailwind CSS via Tauri.
- **Database**: Local SQLite for high-speed metadata and history storage.
- **Footprint**: < 40MB RAM idle usage.

## 🚀 Getting Started

### Prerequisites
- Node.js (v18+)
- Rust (Stable)
- System dependencies (for Ubuntu/Debian):
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
  ```

### Development
1. Clone the repository:
   ```bash
   git clone https://github.com/koustav/crest.git
   cd crest
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

### Building
To build a production bundle (.deb, .AppImage):
```bash
npm run tauri build
```

## 🗺️ Roadmap
- [ ] **Deep AI Integration**: Local LLM support for context-aware commands.
- [ ] **Plugin Marketplace**: A central registry for community extensions.
- [ ] **Mobile Remote**: Control your Linux desktop from your phone via local network.

## 📄 License
MIT © [Koustav](https://github.com/koustav)
