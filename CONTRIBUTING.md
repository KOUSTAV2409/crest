# Contributing to Crest 🏔️

First off, thank you for considering contributing to Crest! It's people like you who make the Linux desktop better for everyone.

## 🌈 Our Philosophy
Crest is built for **Performance**, **Privacy**, and **Pixel-Perfection**. 
- **Performance**: Every millisecond counts. We prefer native Rust for heavy lifting and minimal JS for the UI.
- **Privacy**: No telemetry. No cloud. Local-first is a hard requirement.
- **UI**: We follow the "Premium/Professional" design language. If it doesn't look stunning, it's not ready.

## 🛠️ Getting Started

### 1. Fork and Clone
```bash
git clone https://github.com/YOUR_USERNAME/crest.git
cd crest
```

### 2. Environment Setup
Make sure you have the following installed:
- Rust (Stable)
- Node.js (v18+)
- Tauri CLI (`cargo install tauri-cli`)

Install dependencies:
```bash
npm install
```

### 3. Development Workflow
Use the bundled script so devtools-enabled WebView tooling is linked:

```bash
npm run tauri:dev
```

This runs `tauri dev --features devtools` against the `crest` crate. Releases omit that feature (`npm run build` followed by `tauri build` / CI).

## 📬 Pull Request Process
1. Create a new branch: `git checkout -b feature/your-feature-name`.
2. Keep your PRs focused. If you have multiple features, submit multiple PRs.
3. Ensure your code builds without warnings (`cargo check` and `npm run build`).
4. Update the `README.md` or `DOCUMENTBOOK.md` if you've added new features.

## 🎨 Coding Guidelines

### Rust (The Core)
- Use `snake_case` for variables and functions.
- Prefer `anyhow` for error handling in commands.
- Keep system-level logic separate from Tauri commands in `src-tauri/src/logic`.

### React (The UI)
- Use Functional Components and Hooks.
- Follow the CSS variable system in `index.css`.
- Ensure all components are accessible and keyboard-friendly.

## ⚖️ License
By contributing to Crest, you agree that your contributions will be licensed under the **MIT License**.

Happy hacking! 🚀
