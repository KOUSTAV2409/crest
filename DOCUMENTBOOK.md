# Crest FAQ & Technical Transparency Book

Welcome to the Crest documentation! We believe in building a fast, offline-first, and transparent application launcher. This document aims to answer frequently asked questions and clarify how under-the-hood systems operate so you know exactly what your data and the launcher are doing.

## 🧮 Calculator & Natural Language Math
Crest uses a highly advanced natural language parsing engine (`fend-core`) to evaluate math.

**What it supports:**
- Standard arithmetic: `2+2`, `2^3`
- Natural language: `30 percent of 20`
- Unit conversions: `100 feet in meters`, `5 lbs to kg`
- Base conversions: `0xFF in binary`, `10 in hex`

**Why doesn't it require an `=` prefix?**
Crest evaluates your keystrokes in real-time. If it detects that your search query produces a valid mathematical or conversion result, it seamlessly injects the calculator result at the top of your screen, behaving exactly like a native Spotlight or Raycast experience.

## 💱 Currency Conversion
Crest natively supports live currency exchange conversions (e.g., `1 usd to inr`, `20 eur to gbp`).

**Why is there a slight difference between Crest and Google's exchange rates?**
- **Google Search**: Uses premium, highly expensive financial data feeds (like Morningstar) that update millisecond-by-millisecond.
- **Crest Launcher**: Uses the free public tier of `ExchangeRate-API`. To ensure Crest remains completely free, private, and doesn't spam external servers, Crest fetches the exchange rates in the background once when the app launches and caches them. Because free tier data updates roughly once every 24 hours, you may see minor fluctuations (e.g., a few cents difference) compared to Google's live stock market data.

**Privacy Note:**
Crest **never** sends your typed search queries to the internet. The currency conversion happens 100% locally on your machine using the cached exchange rates.

## 📂 File Searching
Type `/` at the beginning of your query to enter File Mode.

**Examples:**
- `/budget` — finds all files with "budget" in the name
- `/resume` — fuzzy matches any document named resume, cv, etc.
- `/project notes` — finds partial matches across your file names

**Where Crest searches:**
By default, Crest indexes the following directories on startup:
- `~/Documents`
- `~/Downloads`
- `~/Desktop`

**What it skips (for performance):**
- Hidden folders (starting with `.`)
- Developer build artefacts: `node_modules`, `target`, `build`, `dist`, `vendor`, `__pycache__`

**How indexing works:**
When Crest first launches, it runs a background scan of the above directories and caches the file metadata (name, full path, extension) into a local SQLite database. This scan does **not** block app startup — you can start searching for apps and using the calculator immediately while the file index builds in the background. After the first pass, Crest watches those folders (debounced) and re-runs an incremental sync when files change, without wiping the whole index on every save.

**Privacy Note:**
File indexing is 100% local. No filenames or paths ever leave your machine.

**Launching files:**
Pressing `Enter` on a file result will open it with your system's default application (e.g., your PDF viewer, image viewer, or text editor). Crest uses the standard `xdg-open` mechanism under the hood.

## 🌐 Internet Search & URLs
Crest acts as a bridge between your desktop and the web.

**Web Search:**
If your query doesn't match a local app or file, Crest offers a **"Search DuckDuckGo for…"** row. Pressing `Enter` opens your default browser on DuckDuckGo for that query (the same provider used for in-app instant answers and Lite fallbacks).

**In-App Results:**
Crest integrates with DuckDuckGo to fetch real search results and instant answers (like Wikipedia summaries) in the background. After a short pause (600ms) in typing, Crest pulls the top results and displays them directly in the launcher. You can see page titles, descriptions, and site snippets without leaving your keyboard.

**Direct URL Navigation:**
If you type something that looks like a URL (e.g., `github.com`, `localhost:3000`, or `https://...`), Crest will show an **"Open URL"** result at the top. This allows you to jump directly to your favorite sites without manually opening a browser and navigating to the address bar.

**Privacy Note:**
Crest **does not** track your searches. Web results are fetched anonymously using a standard web-scraping fallback when official APIs are unavailable. No user-identifiable tokens, cookies, or account information are ever sent during these requests. Results are only fetched after a "typing pause" to minimize network traffic and respect your privacy.

## ⚙️ Global shortcut & extensions
**Hotkey**: `~/.config/crest/config.json` exposes `global_shortcut` (default `super+Space`). Parsing matches the `global-hotkey` crate (`alt+Esc`, `control+shift+KeyK`, …).

**Super+Space not working on Linux?** If you log in with **Wayland**, the compositor usually does **not** allow the **X11-style global grabs** Crest uses internally. Crest never shows that error as a popup; check **`~/.local/share/crest/hotkey.log`** or launch `crest` from a terminal. **Recommended:** add a desktop shortcut (**Settings → Keyboard → Custom shortcuts**) whose command is **`crest`**. From **v0.2.1** on, Crest is a single instance — each time **`crest`** runs it **toggles** the palette (ideal for Wayland). You can also try **`alt+Space`** in `config.json`, or use an **X11** session (“Ubuntu on Xorg”, etc.), where grabs often succeed.

From **v0.2.2** on, Crest also shows a **yellow hint bar** at the top of the window on affected setups with the same recommendation.

**Plugins**: With the default **`plugin_policy: "manifest"`**, Crest only discovers extensions listed in `~/.config/crest/plugins/manifest.json`. Each entry resolves to a path under that directory—no upward path segments are allowed—and the launcher invokes the binary or script without routing your query through a shell. Scripts should print JSON that deserializes to the same shape as other search rows. Legacy installs may set **`plugin_policy: "open"`** to scan every loose file under the plugins directory (full trust on that folder).
