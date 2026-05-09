use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use once_cell::sync::Lazy;

static EXCHANGE_RATES: Lazy<RwLock<HashMap<String, f64>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn fetch_exchange_rates() {
    if let Ok(resp) = reqwest::get("https://open.er-api.com/v6/latest/USD").await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(rates) = json.get("rates").and_then(|r| r.as_object()) {
                let mut map = HashMap::new();
                for (k, v) in rates {
                    if let Some(f) = v.as_f64() {
                        map.insert(k.to_string(), f);
                    }
                }
                if let Ok(mut cache) = EXCHANGE_RATES.write() {
                    *cache = map;
                }
            }
        }
    }
}

struct CurrencyHandler;
impl fend_core::ExchangeRateFnV2 for CurrencyHandler {
    fn relative_to_base_currency(
        &self,
        currency: &str,
        _options: &fend_core::ExchangeRateFnV2Options,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let cache = EXCHANGE_RATES.read().map_err(|_| "Lock poisoned")?;
        if let Some(&rate) = cache.get(&currency.to_uppercase()) {
            Ok(rate)
        } else {
            Err("Currency not found".into())
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ResultIcon {
    pub kind: String, // "app"|"emoji"|"file"|"url"
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub title: String,
    pub shortcut: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Preview {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: ResultIcon,
    pub category: String,
    pub score: f32,
    pub actions: Vec<Action>,
    pub preview: Option<Preview>,
}

fn sqlite_like_pattern_contains(raw: &str) -> String {
    format!(
        "%{}%",
        raw.trim()
            .to_lowercase()
            .replace('\\', r"\\")
            .replace('%', r"\%")
            .replace('_', r"\_")
    )
}

fn nucleo_run_to_idle<T: Sync + Send + 'static>(matcher: &mut nucleo::Nucleo<T>) {
    // tick may return before the worker finishes; read snapshot only after idle.
    let mut guard = 0u32;
    while matcher.tick(50).running {
        guard += 1;
        if guard >= 500 {
            break;
        }
    }
}

fn search_blocking(query: String, _category: Option<String>) -> Result<Vec<SearchResult>, String> {
    use nucleo::{Config, Nucleo};
    use rusqlite::Connection;
    use std::path::PathBuf;

    let db_path =
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest").join("crest_index.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    struct AppEntry {
        id: String,
        name: String,
        exec: String,
        icon: String,
        comment: String,
    }

    if query.is_empty() {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, exec, icon, comment FROM apps ORDER BY name COLLATE NOCASE LIMIT 80",
            )
            .map_err(|e| e.to_string())?;
        let apps: Vec<AppEntry> = stmt
            .query_map([], |row| {
                Ok(AppEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    exec: row.get(2)?,
                    icon: row.get(3)?,
                    comment: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .take(40)
            .collect();
        return Ok(apps.into_iter().take(20).map(|app| SearchResult {
            id: app.id,
            title: app.name,
            subtitle: app.comment,
            icon: ResultIcon { kind: "app".into(), value: app.icon },
            category: "Applications".into(),
            score: 0.0,
            actions: vec![Action {
                id: "launch".into(),
                title: "Launch".into(),
                shortcut: Some("↵".into()),
            }],
            preview: Some(Preview {
                title: "Launch Application".into(),
                subtitle: Some(app.exec),
                description: None,
            }),
        }).collect());
    }

    let pat = sqlite_like_pattern_contains(&query);
    // One-letter queries match almost every app via `%c%`, which used to load thousands of rows
    // and make Nucleo + IPC feel like a multi-second freeze. Cap + ORDER BY keeps work bounded
    // while still ranking the most relevant slice (name-sorted, then fuzzy on that set).
    let q_chars = query.trim().chars().count() as i64;
    let row_cap: i64 = match q_chars {
        1 => 450,
        2 => 1700,
        _ => 4000,
    };

    let mut stmt = conn
        .prepare(
            r#"SELECT id, name, exec, icon, comment FROM apps
               WHERE lower(name) LIKE ?1 ESCAPE '\'
                  OR lower(COALESCE(comment, '')) LIKE ?1 ESCAPE '\'
               ORDER BY name COLLATE NOCASE
               LIMIT ?2"#,
        )
        .map_err(|e| e.to_string())?;
    let mut apps: Vec<AppEntry> = stmt
        .query_map(rusqlite::params![pat, row_cap], |row| {
            Ok(AppEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                exec: row.get(2)?,
                icon: row.get(3)?,
                comment: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    if apps.is_empty() {
        let mut stmt_full = conn
            .prepare("SELECT id, name, exec, icon, comment FROM apps LIMIT 6500")
            .map_err(|e| e.to_string())?;
        apps = stmt_full
            .query_map([], |row| {
                Ok(AppEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    exec: row.get(2)?,
                    icon: row.get(3)?,
                    comment: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
    }

    let mut matcher = Nucleo::<AppEntry>::new(Config::DEFAULT, std::sync::Arc::new(|| ()), None, 2);
    let injector = matcher.injector();

    for app in apps {
        injector.push(app, |a, columns| {
            columns[0] = a.name.clone().into();
            columns[1] = a.comment.clone().into();
        });
    }

    matcher.pattern.reparse(
        0,
        &query,
        nucleo::pattern::CaseMatching::Ignore,
        nucleo::pattern::Normalization::Smart,
        false,
    );
    nucleo_run_to_idle(&mut matcher);

    let snapshot = matcher.snapshot();
    let count = snapshot.matched_item_count();

    let mut results = Vec::new();
    for i in 0..count.min(20) {
        if let Some(item) = snapshot.get_matched_item(i) {
            let app = item.data;
            results.push(SearchResult {
                id: app.id.clone(),
                title: app.name.clone(),
                subtitle: app.comment.clone(),
                icon: ResultIcon { kind: "app".into(), value: app.icon.clone() },
                category: "Applications".into(),
                score: 0.0,
                actions: vec![Action {
                    id: "launch".into(),
                    title: "Launch".into(),
                    shortcut: Some("↵".into()),
                }],
                preview: Some(Preview {
                    title: app.name.clone(),
                    subtitle: Some(app.exec.clone()),
                    description: Some(app.comment.clone()),
                }),
            });
        }
    }

    let plugins = crate::plugins::list_plugins();
    for plugin in plugins {
        if plugin.name.to_lowercase().contains(&query.to_lowercase()) {
            results.push(SearchResult {
                id: format!("plugin-{}", plugin.name),
                title: plugin.name,
                subtitle: plugin.description,
                icon: ResultIcon {
                    kind: "emoji".into(),
                    value: plugin.icon,
                },
                category: "Extension".into(),
                score: 0.05,
                actions: vec![Action {
                    id: "run_extension".into(),
                    title: "Run".into(),
                    shortcut: Some("↵".into()),
                }],
                preview: None,
            });
        }
    }

    let q_lc = query.to_lowercase();
    if q_lc == "clip" || q_lc == "clipboard" {
        results.push(SearchResult {
            id: "system-clipboard".into(),
            title: "Clipboard History".into(),
            subtitle: "View and search your clipboard history".into(),
            icon: ResultIcon {
                kind: "emoji".into(),
                value: "📋".into(),
            },
            category: "System".into(),
            score: 0.1,
            actions: vec![Action {
                id: "open_clipboard".into(),
                title: "Open".into(),
                shortcut: Some("↵".into()),
            }],
            preview: None,
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn search(query: String, category: Option<String>) -> Result<String, String> {
    let results = tokio::task::spawn_blocking(move || search_blocking(query, category))
        .await
        .map_err(|e| format!("search task join failed: {}", e))??;
        
    let json_bytes = serde_json::to_vec(&results).map_err(|e| e.to_string())?;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    Ok(STANDARD.encode(json_bytes))
}

#[tauri::command]
pub async fn launch_app(app_id: String) -> Result<(), String> {
    use std::process::Command;
    use rusqlite::Connection;
    use std::path::PathBuf;
    
    let db_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest");
    let db_path = db_dir.join("crest_index.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT exec FROM apps WHERE id = ?1").map_err(|e| e.to_string())?;
    let mut exec: String = stmt.query_row([&app_id], |row| row.get(0)).map_err(|e| e.to_string())?;
    
    // Clean up Exec string (remove %U, %f, etc)
    exec = exec.replace("%U", "").replace("%u", "").replace("%F", "").replace("%f", "").trim().to_string();
    
    let args: Vec<&str> = exec.split_whitespace().collect();
    if args.is_empty() {
        return Err("Empty Exec command".into());
    }
    
    Command::new(args[0])
        .args(&args[1..])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
        
    Ok(())
}

#[tauri::command]
pub async fn calculate(expr: String) -> Result<String, String> {
    let mut context = fend_core::Context::new();
    context.set_exchange_rate_handler_v2(CurrencyHandler);
    
    // Map natural language to fend-supported syntax
    let parsed_expr = expr
        .to_lowercase()
        .replace("percent of", "% of")
        .replace("percent", "%")
        .replace("into", "to")
        .replace("dollars", "usd")
        .replace("dollar", "usd")
        .replace("rupees", "inr")
        .replace("rupee", "inr")
        .replace("pounds", "gbp")
        .replace("pound", "gbp")
        .replace("euro", "eur")
        .replace("yen", "jpy");
        
    match fend_core::evaluate(&parsed_expr, &mut context) {
        Ok(result) => {
            let result_str = result.get_main_result();
            if result_str.is_empty() || result_str == expr {
                Err("No meaningful result".to_string())
            } else {
                Ok(result_str.to_string())
            }
        },
        Err(_) => Err("Invalid expression".to_string()),
    }
}

fn search_files_blocking(query: String) -> Result<Vec<SearchResult>, String> {
    use nucleo::{Config, Nucleo};
    use rusqlite::Connection;
    use std::path::PathBuf;

    let db_path =
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest").join("crest_index.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    struct FileEntry {
        id: String,
        name: String,
        path: String,
        extension: String,
    }

    if query.is_empty() {
        let mut stmt = conn
            .prepare("SELECT id, name, path, extension FROM files ORDER BY name COLLATE NOCASE LIMIT 60")
            .map_err(|e| e.to_string())?;
        let files: Vec<FileEntry> = stmt
            .query_map([], |row| {
                Ok(FileEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    extension: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .take(40)
            .collect();
        return Ok(files.into_iter().take(20).map(|file| SearchResult {
            id: file.id,
            title: file.name.clone(),
            subtitle: file.path.clone(),
            icon: ResultIcon { kind: "file".into(), value: file.extension },
            category: "Files".into(),
            score: 0.0,
            actions: vec![Action {
                id: "open_file".into(),
                title: "Open File".into(),
                shortcut: Some("↵".into()),
            }],
            preview: Some(Preview {
                title: file.name,
                subtitle: Some(file.path),
                description: None,
            }),
        }).collect());
    }

    let pat = sqlite_like_pattern_contains(&query);
    let mut stmt = conn
        .prepare(
            r#"SELECT id, name, path, extension FROM files
               WHERE lower(name) LIKE ?1 ESCAPE '\'
                  OR lower(path) LIKE ?1 ESCAPE '\'
               LIMIT 5000"#,
        )
        .map_err(|e| e.to_string())?;
    let mut files: Vec<FileEntry> = stmt
        .query_map([&pat], |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                extension: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    if files.is_empty() {
        let mut stmt_full =
            conn.prepare("SELECT id, name, path, extension FROM files LIMIT 9000")
                .map_err(|e| e.to_string())?;
        files = stmt_full
            .query_map([], |row| {
                Ok(FileEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    extension: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
    }

    let mut matcher = Nucleo::<FileEntry>::new(Config::DEFAULT, std::sync::Arc::new(|| ()), None, 1);
    let injector = matcher.injector();

    for file in files {
        injector.push(file, |f, columns| {
            columns[0] = f.name.clone().into();
        });
    }

    matcher.pattern.reparse(
        0,
        &query,
        nucleo::pattern::CaseMatching::Ignore,
        nucleo::pattern::Normalization::Smart,
        false,
    );
    nucleo_run_to_idle(&mut matcher);

    let snapshot = matcher.snapshot();
    let count = snapshot.matched_item_count();

    let mut results = Vec::new();
    for i in 0..count.min(20) {
        if let Some(item) = snapshot.get_matched_item(i) {
            let file = item.data;
            results.push(SearchResult {
                id: file.id.clone(),
                title: file.name.clone(),
                subtitle: file.path.clone(),
                icon: ResultIcon { kind: "file".into(), value: file.extension.clone() },
                category: "Files".into(),
                score: 0.0,
                actions: vec![Action {
                    id: "open_file".into(),
                    title: "Open File".into(),
                    shortcut: Some("↵".into()),
                }],
                preview: Some(Preview {
                    title: file.name.clone(),
                    subtitle: Some(file.path.clone()),
                    description: None,
                }),
            });
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn search_files(query: String) -> Result<String, String> {
    let results = tokio::task::spawn_blocking(move || search_files_blocking(query))
        .await
        .map_err(|e| format!("search_files task join failed: {}", e))??;
        
    let json_bytes = serde_json::to_vec(&results).map_err(|e| e.to_string())?;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    Ok(STANDARD.encode(json_bytes))
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    use std::process::Command;
    let p = std::path::PathBuf::from(path.trim());
    if p.as_os_str().is_empty() {
        return Err("Empty path".into());
    }
    if !p.is_absolute() {
        return Err("Only absolute paths are allowed".into());
    }
    if !p.exists() {
        return Err("Path does not exist".into());
    }

    Command::new("xdg-open")
        .arg(&p)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("xdg-open failed: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn search_web(query: String) -> Result<(), String> {
    use std::process::Command;
    let url = format!(
        "https://duckduckgo.com/?q={}",
        urlencoding::encode(&query)
    );
    Command::new("xdg-open")
        .arg(&url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("xdg-open failed: {}", e))?;
    Ok(())
}

/// Resolves DuckDuckGo redirect URLs (`uddg=` or protocol-relative links) to a real HTTPS URL for `xdg-open`.
fn decode_duckduckgo_redirect_href(href: &str) -> String {
    let href = href.trim();
    let absolute = if href.starts_with('/') && !href.starts_with("//") {
        format!("https://duckduckgo.com{href}")
    } else if href.starts_with("//") {
        format!("https:{href}")
    } else {
        href.to_string()
    };

    if absolute.contains("uddg=") {
        return absolute
            .split("uddg=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .map(|s| urlencoding::decode(s).unwrap_or_else(|_| s.into()).into_owned())
            .filter(|u| !u.is_empty())
            .unwrap_or(absolute);
    }

    absolute
}

fn scrape_ddg_static_html(html: &str, max_results: usize) -> Vec<SearchResult> {
    use scraper::{Html, Selector};

    let result_blocks =
        Selector::parse("#links .result").expect("selector #links .result");
    let title_sel = Selector::parse(".result__a").expect("selector .result__a");
    let snippet_sel = Selector::parse(".result__snippet").expect("selector .result__snippet");

    let document = Html::parse_document(html);
    let mut out = Vec::new();

    for block in document.select(&result_blocks).take(max_results) {
        let Some(title_el) = block.select(&title_sel).next() else {
            continue;
        };
        let title = title_el.text().collect::<String>().trim().to_string();
        if title.is_empty() {
            continue;
        }

        let href = title_el.value().attr("href").unwrap_or("");
        let real_url = decode_duckduckgo_redirect_href(href);
        if real_url.is_empty() {
            continue;
        }

        let snippet = block
            .select(&snippet_sel)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        out.push(SearchResult {
            id: format!("web-html-{real_url}"),
            title,
            subtitle: if snippet.is_empty() {
                "Web result".into()
            } else {
                snippet.clone()
            },
            icon: ResultIcon {
                kind: "emoji".into(),
                value: "🌐".into(),
            },
            category: "Web Result".into(),
            score: 0.06,
            actions: vec![Action {
                id: "open_url".into(),
                title: "Open in Browser".into(),
                shortcut: Some("↵".into()),
            }],
            preview: Some(Preview {
                title: "Web Result".into(),
                subtitle: Some(real_url.clone()),
                description: Some(if snippet.is_empty() {
                    real_url.clone()
                } else {
                    snippet
                }),
            }),
        });
    }

    out
}

/// Lite / refreshed DDG markup often lays out organic links as `<tr>… <a href="…uddg…">`.
fn scrape_ddg_lite_table_rows(html: &str, max_results: usize) -> Vec<SearchResult> {
    use scraper::{Html, Selector};

    let Ok(tr_sel) = Selector::parse("tr") else {
        return vec![];
    };
    let Ok(a_sel) = Selector::parse(r#"a[href*="uddg"]"#) else {
        return vec![];
    };

    let document = Html::parse_document(html);
    let mut seen_url: HashSet<String> = HashSet::new();
    let mut out = Vec::new();

    for tr in document.select(&tr_sel) {
        if out.len() >= max_results {
            break;
        }
        let Some(anchor) = tr.select(&a_sel).next() else {
            continue;
        };
        let title = anchor.text().collect::<String>().trim().to_string();
        if title.len() < 2 {
            continue;
        }
        let href = anchor.value().attr("href").unwrap_or("");
        let real_url = decode_duckduckgo_redirect_href(href);
        if real_url.is_empty()
            || !real_url.starts_with("http")
            || !seen_url.insert(real_url.clone())
        {
            continue;
        }

        let row_full = tr.text().collect::<String>();
        let mut snippet = row_full.replace(&title, "");
        snippet = snippet.split_whitespace().collect::<Vec<_>>().join(" ");
        snippet = snippet.trim().trim_matches('|').trim().chars().take(260).collect();
        if snippet.len() < 12 {
            snippet = "Web result".into();
        }

        out.push(SearchResult {
            id: format!("web-ddg-{real_url}"),
            title,
            subtitle: snippet.clone(),
            icon: ResultIcon {
                kind: "emoji".into(),
                value: "🌐".into(),
            },
            category: "Web Result".into(),
            score: 0.055,
            actions: vec![Action {
                id: "open_url".into(),
                title: "Open in Browser".into(),
                shortcut: Some("↵".into()),
            }],
            preview: Some(Preview {
                title: "Web Result".into(),
                subtitle: Some(real_url.clone()),
                description: Some(snippet.clone()),
            }),
        });
    }

    out
}

/// Collect `uddg` links when markup has no recognizable table / `.result__a` wrappers.
fn scrape_ddg_fallback_anchors(html: &str, max_results: usize) -> Vec<SearchResult> {
    use scraper::{Html, Selector};

    let Ok(a_sel) = Selector::parse(r#"a[href*="uddg"]"#) else {
        return vec![];
    };
    let document = Html::parse_document(html);
    let mut seen_url: HashSet<String> = HashSet::new();
    let mut out = Vec::new();

    for a in document.select(&a_sel).take(max_results * 4) {
        if out.len() >= max_results {
            break;
        }
        let title = a.text().collect::<String>().trim().to_string();
        if title.len() < 3 {
            continue;
        }
        let href = a.value().attr("href").unwrap_or("");
        let real_url = decode_duckduckgo_redirect_href(href);
        if real_url.is_empty()
            || !real_url.starts_with("http")
            || !seen_url.insert(real_url.clone())
        {
            continue;
        }
        out.push(SearchResult {
            id: format!("web-a-{real_url}"),
            title,
            subtitle: "Web result".into(),
            icon: ResultIcon {
                kind: "emoji".into(),
                value: "🌐".into(),
            },
            category: "Web Result".into(),
            score: 0.048,
            actions: vec![Action {
                id: "open_url".into(),
                title: "Open in Browser".into(),
                shortcut: Some("↵".into()),
            }],
            preview: Some(Preview {
                title: "Web Result".into(),
                subtitle: Some(real_url.clone()),
                description: Some(real_url.clone()),
            }),
        });
    }

    out
}

fn scrape_ddg_html_bundle(html: &str, max_results: usize) -> Vec<SearchResult> {
    let strategies: [fn(&str, usize) -> Vec<SearchResult>; 3] = [
        scrape_ddg_lite_table_rows,
        scrape_ddg_static_html,
        scrape_ddg_fallback_anchors,
    ];
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for strat in strategies {
        for row in strat(html, max_results) {
            if row.category != "Web Result" {
                continue;
            }
            let Some(url) = row.preview.as_ref().and_then(|p| p.subtitle.as_deref()) else {
                continue;
            };
            if url.len() < 8 || !seen.insert(url.to_string()) {
                continue;
            }
            out.push(row);
            if out.len() >= max_results {
                return out;
            }
        }
    }
    out
}

fn merge_web_unique(target: &mut Vec<SearchResult>, add: Vec<SearchResult>) {
    let mut seen: HashSet<String> = target
        .iter()
        .filter(|r| r.category == "Web Result")
        .filter_map(|r| r.preview.as_ref()?.subtitle.clone())
        .collect();

    for row in add {
        if row.category != "Web Result" {
            continue;
        }
        let Some(url) = row.preview.as_ref().and_then(|p| p.subtitle.clone()) else {
            continue;
        };
        if url.len() < 8 || !seen.insert(url) {
            continue;
        }
        target.push(row);
    }
}

/// DuckDuckGo serves a CAPTCHA (“bots use DuckDuckGo too”) instead of SERP HTML for many non-browser clients.
fn ddg_html_looks_like_challenge(html: &str) -> bool {
    html.contains("anomaly-modal")
        || html.contains("bots use DuckDuckGo")
        || html.contains("challenge-form")
}

fn strip_html_tags_light(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&#039;", "'")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&ndash;", "–")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// MediaWiki [`list=search`](https://www.mediawiki.org/wiki/API:Search) — works when DDG HTML is bot-blocked.
async fn fetch_wikipedia_search_results(
    client: &reqwest::Client,
    query: &str,
    max: usize,
) -> Vec<SearchResult> {
    let lim = max.clamp(1, 15);
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&format=json&list=search&utf8=1&srlimit={lim}&srsearch={}",
        urlencoding::encode(query)
    );

    let Ok(resp) = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::USER_AGENT,
            concat!(
                "Crest/",
                env!("CARGO_PKG_VERSION"),
                " (Tauri launcher; https://tauri.app)"
            ),
        )
        .send()
        .await
    else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    let Ok(json) = resp.json::<serde_json::Value>().await else {
        return Vec::new();
    };

    let Some(arr) = json
        .get("query")
        .and_then(|q| q.get("search"))
        .and_then(|s| s.as_array())
    else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for item in arr.iter().take(lim) {
        let Some(title) = item.get("title").and_then(|v| v.as_str()).map(str::trim) else {
            continue;
        };
        if title.is_empty() {
            continue;
        }
        let snippet_raw = item.get("snippet").and_then(|v| v.as_str()).unwrap_or("");
        let snippet = strip_html_tags_light(snippet_raw);
        let subtitle = if snippet.len() >= 16 {
            snippet.clone()
        } else {
            "Wikipedia article".into()
        };

        let Some(pageid) = item.get("pageid").and_then(|v| v.as_u64()).or_else(|| {
            item.get("pageid")
                .and_then(|v| v.as_i64())
                .filter(|&p| p > 0)
                .map(|p| p as u64)
        }) else {
            continue;
        };

        let page_url = format!("https://en.wikipedia.org/?curid={pageid}");

        out.push(SearchResult {
            id: format!("wiki-{pageid}"),
            title: title.to_string(),
            subtitle,
            icon: ResultIcon {
                kind: "emoji".into(),
                value: "📚".into(),
            },
            category: "Web Result".into(),
            score: 0.07,
            actions: vec![Action {
                id: "open_url".into(),
                title: "Open in Browser".into(),
                shortcut: Some("↵".into()),
            }],
            preview: Some(Preview {
                title: "Wikipedia".into(),
                subtitle: Some(page_url.clone()),
                description: Some(if snippet.len() >= 16 {
                    snippet
                } else {
                    title.to_string()
                }),
            }),
        });
    }

    if !out.is_empty() {
        println!(
            "(wikipedia) added {} encyclopedia hits",
            out.len()
        );
    }

    out
}

fn append_related_topics_from_api(json: &serde_json::Value, results: &mut Vec<SearchResult>, budget: usize) {
    fn walk(value: &serde_json::Value, results: &mut Vec<SearchResult>, budget: usize) {
        if results.len() >= budget {
            return;
        }

        match value {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    walk(item, results, budget);
                    if results.len() >= budget {
                        return;
                    }
                }
            }
            serde_json::Value::Object(map) => {
                if let Some(nested) = map.get("Topics").and_then(|v| v.as_array()) {
                    for item in nested {
                        walk(item, results, budget);
                        if results.len() >= budget {
                            return;
                        }
                    }
                }

                let Some(text) = map.get("Text").and_then(|v| v.as_str()).map(str::trim) else {
                    return;
                };
                if text.is_empty() {
                    return;
                }
                let Some(url) = map.get("FirstURL").and_then(|v| v.as_str()).map(str::trim) else {
                    return;
                };
                if url.is_empty() {
                    return;
                }

                let id = format!("web-topic-{url}");
                if results.iter().any(|r| r.id == id) {
                    return;
                }

                let (title, subtitle) = if let Some(idx) = text.find(" - ") {
                    let t = text[..idx].trim();
                    let s = text[idx + 3..].trim();
                    if t.is_empty() {
                        (text.to_string(), String::new())
                    } else {
                        (t.to_string(), s.to_string())
                    }
                } else {
                    (text.to_string(), String::new())
                };

                results.push(SearchResult {
                    id,
                    title,
                    subtitle: if subtitle.is_empty() {
                        "Related topic".into()
                    } else {
                        subtitle
                    },
                    icon: ResultIcon {
                        kind: "emoji".into(),
                        value: "💡".into(),
                    },
                    category: "Web Answer".into(),
                    score: 0.72,
                    actions: vec![Action {
                        id: "open_url".into(),
                        title: "Open Link".into(),
                        shortcut: Some("↵".into()),
                    }],
                    preview: Some(Preview {
                        title: "Related".into(),
                        subtitle: Some(url.to_string()),
                        description: Some(text.to_string()),
                    }),
                });
            }
            _ => {}
        }
    }

    if let Some(topics) = json.get("RelatedTopics") {
        walk(topics, results, budget);
    }
}

#[tauri::command]
pub async fn fetch_web_results(query: String) -> Result<Vec<SearchResult>, String> {
    println!("Web search requested for: {}", query);
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;
    
    let mut results = Vec::new();

    // 1. Try DuckDuckGo Instant Answer API (JSON)
    let api_url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1", urlencoding::encode(&query));
    if let Ok(resp) = client.get(&api_url).send().await {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            let abstract_text = json.get("AbstractText").and_then(|v| v.as_str()).unwrap_or("");
            if !abstract_text.is_empty() {
                println!("Found JSON Abstract: {}", abstract_text);
                let abstract_source = json.get("AbstractSource").and_then(|v| v.as_str()).unwrap_or("Wikipedia");
                let abstract_url = json.get("AbstractURL").and_then(|v| v.as_str()).unwrap_or("");

                results.push(SearchResult {
                    id: format!("web-abs-{}", abstract_url),
                    title: abstract_text.to_string(),
                    subtitle: format!("Source: {}", abstract_source),
                    icon: ResultIcon { kind: "emoji".into(), value: "💡".into() },
                    category: "Web Answer".into(),
                    score: 0.9,
                    actions: vec![
                        Action { id: "open_url".into(), title: "Read More".into(), shortcut: Some("↵".into()) },
                        Action { id: "copy".into(), title: "Copy Answer".into(), shortcut: Some("⌘C".into()) }
                    ],
                    preview: Some(Preview {
                        title: "Instant Answer".into(),
                        subtitle: Some(abstract_url.to_string()),
                        description: Some(abstract_text.to_string()),
                    })
                });
            }

            append_related_topics_from_api(&json, &mut results, 8);
        }
    }

    // 2. DuckDuckGo HTML SERP (GET + POST) — parsers include lite-style `<tr>` rows with `uddg` links.
    if results.iter().filter(|r| r.category == "Web Result").count() < 3 {
        const MAX_ORGANIC: usize = 10;
        let html_url = format!(
            "https://html.duckduckgo.com/html/?q={}&kl=us-en",
            urlencoding::encode(&query)
        );
        println!("Fetching DuckDuckGo HTML SERP (GET)…");

        match client
            .get(&html_url)
            .header("Accept-Language", "en-US,en;q=0.9")
            .header(
                reqwest::header::ACCEPT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(html) = resp.text().await {
                    if ddg_html_looks_like_challenge(&html) {
                        println!(
                            "(html GET) DuckDuckGo returned a bot challenge page; skipping HTML scrape"
                        );
                    } else {
                        let scraped = scrape_ddg_html_bundle(&html, MAX_ORGANIC);
                        println!(
                            "(html GET) organic bundle scraped {} hits",
                            scraped.len()
                        );
                        if scraped.is_empty() {
                            let sample: String =
                                html.chars().take(520).collect::<String>().replace('\n', " ");
                            println!("(html GET) diagnostic (trimmed): {sample}");
                        }
                        merge_web_unique(&mut results, scraped);
                    }
                }
            }
            Ok(resp) => println!("(html GET) HTTP {}", resp.status()),
            Err(e) => println!("(html GET) request failed: {}", e),
        }
    }

    if results.iter().filter(|r| r.category == "Web Result").count() < 2 {
        println!("Fetching DuckDuckGo HTML SERP (POST fallback)…");
        match client
            .post("https://html.duckduckgo.com/html/")
            .header(reqwest::header::REFERER, "https://html.duckduckgo.com/")
            .header("Accept-Language", "en-US,en;q=0.9")
            .form(&[("q", query.as_str()), ("b", "")])
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(html) = resp.text().await {
                    if !ddg_html_looks_like_challenge(&html) {
                        merge_web_unique(&mut results, scrape_ddg_html_bundle(&html, 10));
                    } else {
                        println!("(html POST) bot challenge page; skipping scrape");
                    }
                }
            }
            Err(e) => println!("(html POST) failed: {}", e),
            Ok(resp) => println!("(html POST) HTTP {}", resp.status()),
        }
    }

    // 3. `/lite` table SERP
    if results.iter().filter(|r| r.category == "Web Result").count() < 3 {
        let lite_url = format!(
            "https://duckduckgo.com/lite/?q={}",
            urlencoding::encode(&query)
        );
        println!("Fetching duckduckgo.com/lite…");
        match client
            .get(&lite_url)
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(html) = resp.text().await {
                    if ddg_html_looks_like_challenge(&html) {
                        println!("(lite) bot challenge page; skipping scrape");
                    } else {
                        let scraped = scrape_ddg_html_bundle(&html, 10);
                        let organic_hits = scraped.len();
                        println!("(lite) organic bundle scraped {organic_hits} hits");
                        merge_web_unique(&mut results, scraped);
                        if organic_hits == 0 && !html.is_empty() {
                            let sample: String =
                                html.chars().take(520).collect::<String>().replace('\n', " ");
                            println!("(lite) diagnostic (trimmed): {sample}");
                        }
                    }
                }
            }
            Ok(resp) => println!("(lite) HTTP {}", resp.status()),
            Err(e) => println!("(lite) request failed: {}", e),
        }
    }

    let organic_web = results.iter().filter(|r| r.category == "Web Result").count();
    if organic_web < 3 {
        println!(
            "Augmenting with Wikipedia site search (organic DuckDuckGo rows: {organic_web})…"
        );
        merge_web_unique(
            &mut results,
            fetch_wikipedia_search_results(&client, &query, 10).await,
        );
    }

    if results.is_empty() {
        println!("No results found even with fallback for: {}", query);
    } else {
        println!("Returning {} results", results.len());
    }
    
    Ok(results)
}
#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    use std::process::Command;
    let u = url.trim();
    if u.is_empty() {
        return Err("Empty URL".into());
    }
    // Basic scheme allowlist. Avoids opening `file:`, `smb:`, `data:`, etc.
    // Prefer handling broader cases via `tauri-plugin-opener` scopes.
    let u_lc = u.to_ascii_lowercase();
    if !(u_lc.starts_with("https://") || u_lc.starts_with("http://")) {
        return Err("Only http(s) URLs are allowed".into());
    }
    if u_lc.chars().any(|c| c.is_control()) {
        return Err("URL contains invalid control characters".into());
    }

    Command::new("xdg-open")
        .arg(u)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("xdg-open failed: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn quit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}
