use serde::{Serialize, Deserialize};
use std::sync::RwLock;
use std::collections::HashMap;
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

#[tauri::command]
pub async fn search(query: String, _category: Option<String>) -> Result<Vec<SearchResult>, String> {
    use rusqlite::Connection;
    use nucleo::{Nucleo, Config};
    use std::path::PathBuf;
    
    let db_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest");
    let db_path = db_dir.join("crest_index.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, name, exec, icon, comment FROM apps").map_err(|e| e.to_string())?;
    
    struct AppEntry {
        id: String,
        name: String,
        exec: String,
        icon: String,
        comment: String,
    }
    
    let app_iter = stmt.query_map([], |row| {
        Ok(AppEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            exec: row.get(2)?,
            icon: row.get(3)?,
            comment: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut apps = Vec::new();
    for app in app_iter {
        if let Ok(app) = app {
            apps.push(app);
        }
    }
    
    if query.is_empty() {
        // Return top apps if query is empty
        return Ok(apps.into_iter().take(20).map(|app| SearchResult {
            id: app.id,
            title: app.name,
            subtitle: app.comment,
            icon: ResultIcon { kind: "app".into(), value: app.icon },
            category: "Applications".into(),
            score: 0.0,
            actions: vec![
                Action { id: "launch".into(), title: "Launch".into(), shortcut: Some("↵".into()) }
            ],
            preview: Some(Preview {
                title: "Launch Application".into(),
                subtitle: Some(app.exec),
                description: None,
            })
        }).collect());
    }
    
    // Setup Nucleo fuzzy matcher
    let mut matcher = Nucleo::<AppEntry>::new(Config::DEFAULT, std::sync::Arc::new(|| ()), None, 2);
    let injector = matcher.injector();
    
    for app in apps {
        injector.push(app, |a, columns| {
            columns[0] = a.name.clone().into();
            columns[1] = a.comment.clone().into();
        });
    }
    
    matcher.pattern.reparse(0, &query, nucleo::pattern::CaseMatching::Ignore, nucleo::pattern::Normalization::Smart, false);
    matcher.tick(10); // Run matcher with timeout
    
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
                score: 0.0, // Score not easily available in 0.5 without SnapshotItem extensions
                actions: vec![
                    Action { id: "launch".into(), title: "Launch".into(), shortcut: Some("↵".into()) }
                ],
                preview: Some(Preview {
                    title: app.name.clone(),
                    subtitle: Some(app.exec.clone()),
                    description: Some(app.comment.clone()),
                })
            });
        }
    }
    
    Ok(results)
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

#[tauri::command]
pub async fn search_files(query: String) -> Result<Vec<SearchResult>, String> {
    use rusqlite::Connection;
    use nucleo::{Nucleo, Config};
    use std::path::PathBuf;
    
    let db_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest");
    let db_path = db_dir.join("crest_index.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, name, path, extension FROM files").map_err(|e| e.to_string())?;
    
    struct FileEntry {
        id: String,
        name: String,
        path: String,
        extension: String,
    }
    
    let file_iter = stmt.query_map([], |row| {
        Ok(FileEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            extension: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut files = Vec::new();
    for file in file_iter {
        if let Ok(file) = file {
            files.push(file);
        }
    }
    
    if query.is_empty() {
        return Ok(files.into_iter().take(20).map(|file| SearchResult {
            id: file.id,
            title: file.name.clone(),
            subtitle: file.path.clone(),
            icon: ResultIcon { kind: "file".into(), value: file.extension },
            category: "Files".into(),
            score: 0.0,
            actions: vec![
                Action { id: "open_file".into(), title: "Open File".into(), shortcut: Some("↵".into()) }
            ],
            preview: Some(Preview {
                title: file.name,
                subtitle: Some(file.path),
                description: None,
            })
        }).collect());
    }
    
    // Setup Nucleo fuzzy matcher
    let mut matcher = Nucleo::<FileEntry>::new(Config::DEFAULT, std::sync::Arc::new(|| ()), None, 1);
    let injector = matcher.injector();
    
    for file in files {
        injector.push(file, |f, columns| {
            columns[0] = f.name.clone().into();
        });
    }
    
    matcher.pattern.reparse(0, &query, nucleo::pattern::CaseMatching::Ignore, nucleo::pattern::Normalization::Smart, false);
    matcher.tick(10); // Run matcher with timeout
    
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
                actions: vec![
                    Action { id: "open_file".into(), title: "Open File".into(), shortcut: Some("↵".into()) }
                ],
                preview: Some(Preview {
                    title: file.name.clone(),
                    subtitle: Some(file.path.clone()),
                    description: None,
                })
            });
        }
    }
    
    Ok(results)
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    use std::process::Command;
    Command::new("xdg-open")
        .arg(&path)
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
    let url = format!("https://www.google.com/search?q={}", urlencoding::encode(&query));
    Command::new("xdg-open")
        .arg(&url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("xdg-open failed: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn fetch_web_results(query: String) -> Result<Vec<SearchResult>, String> {
    println!("Web search requested for: {}", query);
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
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
        }
    }

    // 2. Fallback to scraping DuckDuckGo Lite for organic results
    if results.len() < 3 {
        println!("Falling back to Lite scraping for: {}", query);
        let lite_url = format!("https://duckduckgo.com/lite/?q={}", urlencoding::encode(&query));
        if let Ok(resp) = client.get(&lite_url).send().await {
            if let Ok(html) = resp.text().await {
                let document = scraper::Html::parse_document(&html);
                let result_selector = scraper::Selector::parse(".result-link").unwrap();
                let snippet_selector = scraper::Selector::parse(".result-snippet").unwrap();
                
                // Lite results are often structured in a way that selectors need to be precise
                // Actually, Lite uses simple tables or divs.
                // Let's use a more general selector if the above fails
                let table_selector = scraper::Selector::parse("table").unwrap();
                let rows = document.select(&table_selector).nth(2); // The main results table is usually the 3rd one
                
                if let Some(table) = rows {
                    let link_selector = scraper::Selector::parse("a.result-link").unwrap();
                    let snippet_selector = scraper::Selector::parse("td.result-snippet").unwrap();
                    
                    let links: Vec<_> = table.select(&link_selector).take(5).collect();
                    let snippets: Vec<_> = table.select(&snippet_selector).take(5).collect();
                    
                    for (i, link_node) in links.into_iter().enumerate() {
                        let title = link_node.text().collect::<String>().trim().to_string();
                        let href = link_node.value().attr("href").unwrap_or("");
                        
                        // Parse the real URL from DDG redirect
                        let real_url = if href.contains("uddg=") {
                            href.split("uddg=").nth(1)
                                .and_then(|s| s.split("&").next())
                                .map(|s| urlencoding::decode(s).unwrap_or(s.into()).into_owned())
                                .unwrap_or_else(|| href.to_string())
                        } else {
                            href.to_string()
                        };

                        let snippet = snippets.get(i)
                            .map(|s| s.text().collect::<String>().trim().to_string())
                            .unwrap_or_else(|| "No description available".to_string());

                        results.push(SearchResult {
                            id: format!("web-lite-{}", real_url),
                            title,
                            subtitle: snippet.clone(),
                            icon: ResultIcon { kind: "emoji".into(), value: "🌐".into() },
                            category: "Web Result".into(),
                            score: 0.05,
                            actions: vec![
                                Action { id: "open_url".into(), title: "Open in Browser".into(), shortcut: Some("↵".into()) }
                            ],
                            preview: Some(Preview {
                                title: "Web Result".into(),
                                subtitle: Some(real_url),
                                description: Some(snippet),
                            })
                        });
                    }
                }
            }
        }
    }
    
    if results.is_empty() {
        println!("No results found even with fallback for: {}", query);
    } else {
        println!("Returning {} results", results.len());
    }
    
    Ok(results)
}
