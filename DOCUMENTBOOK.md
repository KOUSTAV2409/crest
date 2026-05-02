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
When Crest first launches, it runs a background scan of the above directories and caches the file metadata (name, full path, extension) into a local SQLite database. This scan does **not** block app startup — you can start searching for apps and using the calculator immediately while the file index builds in the background.

**Privacy Note:**
File indexing is 100% local. No filenames or paths ever leave your machine.

**Launching files:**
Pressing `Enter` on a file result will open it with your system's default application (e.g., your PDF viewer, image viewer, or text editor). Crest uses the standard `xdg-open` mechanism under the hood.

## 🌐 Internet Search & URLs
Crest acts as a bridge between your desktop and the web.

**Web Search:**
If your query doesn't match a local app or file, Crest automatically offers a **"Search Google for..."** option at the bottom of the list. Pressing `Enter` on this will instantly open your default web browser and perform the search.

**Direct URL Navigation:**
If you type something that looks like a URL (e.g., `github.com`, `localhost:3000`, or `https://...`), Crest will show an **"Open URL"** result at the top. This allows you to jump directly to your favorite sites without manually opening a browser and navigating to the address bar.

**Privacy Note:**
Crest **does not** send your keystrokes to any search engine as you type. The "Search Google" option is generated entirely locally. Your query is only sent to the browser when you explicitly press `Enter` on the search result.
