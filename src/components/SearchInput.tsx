import React, { useEffect, useRef, startTransition } from 'react';
import { Search } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store';
import type { SearchResult } from '../types/ipc';
import './SearchInput.css';

const WEB_SEARCH_DEBOUNCE_MS = 600;

/** Minimum delay before kicking off IPC so bursts like "CEO" do not overlap heavy work every key. */
const MIN_SEARCH_DEBOUNCE_MS = 64;

/** Steady-state debounce once the query is longer. */
const DEFAULT_SEARCH_DEBOUNCE_MS = 105;

/** Avoid `calculate` IPC for arbitrary short strings like "c" or app names. */
function mightBeQuickCalc(expr: string): boolean {
  const t = expr.trim();
  if (t.length === 0) return false;
  if (t.length === 1) return /\d/.test(t);
  return /^[\d\s+\-*/().,]+$/.test(t);
}

const WEB_RESULT_CATEGORIES = new Set(['Web Result', 'Web Answer']);

function asSearchResults(value: unknown): SearchResult[] {
  return Array.isArray(value) ? (value as SearchResult[]) : [];
}

// Fast Base64 to JSON Decoder for high-performance IPC
function decodeBase64Results(base64Str: string): SearchResult[] {
  const binaryString = atob(base64Str);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  const jsonStr = new TextDecoder().decode(bytes);
  return JSON.parse(jsonStr) as SearchResult[];
}

const SearchInput: React.FC = () => {
  const setQuery = useAppStore((s) => s.setQuery);
  const setMode = useAppStore((s) => s.setMode);
  const setResults = useAppStore((s) => s.setResults);
  const mode = useAppStore((s) => s.mode);

  const inputRef = useRef<HTMLInputElement>(null);
  const webSearchTimeoutRef = useRef<number | null>(null);
  const defaultSearchTimeoutRef = useRef<number | null>(null);

  const [defaultValue] = React.useState(() => useAppStore.getState().query);

  useEffect(() => {
    return useAppStore.subscribe((state) => {
      const el = inputRef.current;
      if (!el) return;
      const q = state.query;
      if (el.value !== q) {
        el.value = q;
      }
    });
  }, []);

  const modeLabel: Record<string, string> = {
    default: 'apps and commands',
    calculator: 'calculator',
    command: 'commands',
    file: 'files',
    clipboard: 'clipboard history',
  };

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    return () => {
      if (webSearchTimeoutRef.current !== null) {
        window.clearTimeout(webSearchTimeoutRef.current);
      }
      if (defaultSearchTimeoutRef.current !== null) {
        window.clearTimeout(defaultSearchTimeoutRef.current);
      }
    };
  }, []);

  const currentPlaceholder =
    mode === 'default'
      ? 'Search for apps and commands...'
      : `Search in ${modeLabel[mode]}...`;

  const isUrl = (str: string) => {
    try {
      const url = str.trim();
      return (
        url.startsWith('http://') ||
        url.startsWith('https://') ||
        /^([a-z0-9]+(-[a-z0-9]+)*\.)+[a-z]{2,}(:\d+)?(\/.*)?$/i.test(url)
      );
    } catch {
      return false;
    }
  };

  const runAfterInput = (val: string) => {
    setQuery(val);

    const cancelDefaultSearchDebounce = () => {
      if (defaultSearchTimeoutRef.current !== null) {
        window.clearTimeout(defaultSearchTimeoutRef.current);
        defaultSearchTimeoutRef.current = null;
      }
    };

    if (val.startsWith('=')) {
      cancelDefaultSearchDebounce();
      setMode('calculator');
      if (val.length > 1) {
        void (async () => {
          try {
            const res = await invoke<string>('calculate', { expr: val.substring(1) });
            const row: SearchResult = {
              id: 'calc',
              title: res,
              subtitle: 'Result',
              category: 'Calculator',
              icon: { kind: 'emoji', value: '🧮' },
              score: 1,
              actions: [{ id: 'copy', title: 'Copy Result', shortcut: '↵' }],
            };
            startTransition(() => setResults([row]));
          } catch {
            /* invalid expression */
          }
        })();
      } else {
        startTransition(() => setResults([]));
      }
      return;
    }

    if (val.startsWith('>')) {
      cancelDefaultSearchDebounce();
      setMode('terminal');
      const cmd = val.substring(1).trim();
      startTransition(() => {
        setResults([{
          id: `term-${cmd}`,
          title: 'Interactive Shell',
          subtitle: cmd ? `Execute: ${cmd}` : 'Launch a persistent native terminal session',
          category: 'Terminal',
          icon: { kind: 'emoji', value: '>_' },
          score: 1,
          actions: [{ id: 'run_terminal', title: 'Start', shortcut: '↵' }],
        }]);
      });
      return;
    }

    if (val.startsWith('/')) {
      cancelDefaultSearchDebounce();
      setMode('file');
      void (async () => {
        try {
          const queryStr = val === '/' ? '' : val.substring(1);
          const res = asSearchResults(
            decodeBase64Results(await invoke<string>('search_files', { query: queryStr }))
          );
          startTransition(() => setResults(res));
        } catch (e) {
          console.error('File search error', e);
        }
      })();
      return;
    }

    if (mode === 'clipboard') {
      cancelDefaultSearchDebounce();
      void (async () => {
        try {
          const res = asSearchResults(
            await invoke<unknown>('get_clipboard_history')
          );
          const v = val.toLowerCase();
          const filtered = res.filter(
            (item) =>
              item.title.toLowerCase().includes(v) ||
              (item.preview?.description?.toLowerCase().includes(v) ?? false)
          );
          startTransition(() => setResults(filtered));
        } catch (e) {
          console.error('Clipboard fetch error', e);
        }
      })();
      return;
    }

    setMode('default');
    if (val.trim() === '') {
      cancelDefaultSearchDebounce();
      if (webSearchTimeoutRef.current !== null) {
        window.clearTimeout(webSearchTimeoutRef.current);
        webSearchTimeoutRef.current = null;
      }
      startTransition(() => setResults([]));
      return;
    }

    if (val.length <= 2 && webSearchTimeoutRef.current !== null) {
      window.clearTimeout(webSearchTimeoutRef.current);
      webSearchTimeoutRef.current = null;
    }

    cancelDefaultSearchDebounce();
    const capturedQuery = val;
    const debounceMs =
      capturedQuery.trim().length <= 2
        ? MIN_SEARCH_DEBOUNCE_MS
        : DEFAULT_SEARCH_DEBOUNCE_MS;

    defaultSearchTimeoutRef.current = window.setTimeout(() => {
      defaultSearchTimeoutRef.current = null;
      void (async () => {
        try {
          if (useAppStore.getState().query !== capturedQuery) return;

          let res = asSearchResults(
            decodeBase64Results(await invoke<string>('search', { query: capturedQuery, category: null }))
          );

          if (useAppStore.getState().query !== capturedQuery) return;

          if (mightBeQuickCalc(capturedQuery)) {
            try {
              const calcRes = await invoke<string>('calculate', { expr: capturedQuery });
              if (useAppStore.getState().query !== capturedQuery) return;

              let calcResStr = calcRes.replace(/(\.\d{2})\d+/, '$1');

              const row: SearchResult = {
                id: 'calc',
                title: calcResStr,
                subtitle: 'Calculator Result',
                category: 'Calculator',
                icon: { kind: 'emoji', value: '🧮' },
                score: 1.1,
                actions: [{ id: 'copy', title: 'Copy Result', shortcut: '↵' }],
              };
              res = [row, ...res];
            } catch {
              /* not a calculator query */
            }
          }

          if (useAppStore.getState().query !== capturedQuery) return;

          const webRow: SearchResult = {
            id: `web-search-${capturedQuery}`,
            title: `Search DuckDuckGo for "${capturedQuery}"`,
            subtitle: 'Opens in your default browser',
            category: 'Internet',
            icon: { kind: 'emoji', value: '🔍' },
            score: 0.1,
            actions: [{ id: 'search_web', title: 'Search DuckDuckGo', shortcut: '↵' }],
          };
          res = [...res, webRow];

          if (isUrl(capturedQuery)) {
            const url = capturedQuery.startsWith('http')
              ? capturedQuery
              : `https://${capturedQuery}`;
            const openRow: SearchResult = {
              id: `open-url-${url}`,
              title: `Open ${url}`,
              subtitle: 'Web Browser',
              category: 'Internet',
              icon: { kind: 'emoji', value: '🌐' },
              score: 1.2,
              actions: [{ id: 'open_url', title: 'Open in Browser', shortcut: '↵' }],
            };
            res = [openRow, ...res];
          }

          startTransition(() => setResults(res));

          if (capturedQuery.length > 2) {
            const currentQuery = capturedQuery;
            if (webSearchTimeoutRef.current !== null) {
              window.clearTimeout(webSearchTimeoutRef.current);
            }
            webSearchTimeoutRef.current = window.setTimeout(async () => {
              try {
                const webResults = asSearchResults(
                  await invoke<unknown>('fetch_web_results', { query: currentQuery })
                );

                const { query: latestQuery, results: currentResults } = useAppStore.getState();

                if (webResults.length > 0 && latestQuery === currentQuery) {
                  const nonWebResults = currentResults.filter(
                    (r) => !WEB_RESULT_CATEGORIES.has(r.category)
                  );
                  startTransition(() => {
                    setResults([...nonWebResults, ...webResults]);
                  });
                }
              } catch (e) {
                console.error('Web fetch error', e);
              }
            }, WEB_SEARCH_DEBOUNCE_MS);
          }
        } catch (err) {
          console.error('Search error', err);
        }
      })();
    }, debounceMs);
  };

  return (
    <div className="search-input-container">
      <div className="search-field">
        <div className="search-icon" aria-hidden="true">
          <Search size={18} strokeWidth={2.5} />
        </div>
        <input
          ref={inputRef}
          type="text"
          defaultValue={defaultValue}
          onChange={(e) => runAfterInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Backspace' && mode !== 'default') {
              requestAnimationFrame(() => {
                const v = inputRef.current?.value ?? '';
                if (v !== '') return;
                startTransition(() => setMode('default'));
                startTransition(() => setResults([]));
              });
            } else if (e.key === 'ArrowDown') {
              e.preventDefault();
              const { results, activeIndex, setActiveIndex } = useAppStore.getState();
              if (results.length > 0) {
                setActiveIndex((activeIndex + 1) % results.length);
              }
            } else if (e.key === 'ArrowUp') {
              e.preventDefault();
              const { results, activeIndex, setActiveIndex } = useAppStore.getState();
              if (results.length > 0) {
                setActiveIndex((activeIndex - 1 + results.length) % results.length);
              }
            } else if (e.key === 'Enter') {
              const { results, activeIndex } = useAppStore.getState();
              const item = results[activeIndex];
              if (item) {
                const activeEl = document.querySelector('.result-item.active') as HTMLElement;
                activeEl?.click();
              }
            }
          }}
          placeholder={currentPlaceholder}
          className="search-input"
          id="main-search-input"
          spellCheck={false}
          autoComplete="off"
          autoCorrect="off"
        />
        <div className="search-badges" aria-hidden="true">
          <div className="search-kbd-group">
            <kbd className="search-kbd-badge">Ctrl</kbd>
            <kbd className="search-kbd-badge">K</kbd>
          </div>
        </div>
      </div>

      {mode === 'file' && <span className="search-mode-pill mode-file">📂 Files</span>}
      {mode === 'calculator' && <span className="search-mode-pill mode-calc">🧮 Calc</span>}
      {mode === 'command' && <span className="search-mode-pill mode-cmd">Commands</span>}
      {mode === 'clipboard' && <span className="search-mode-pill mode-clip">📋 Clip</span>}
    </div>
  );
};

export default SearchInput;
