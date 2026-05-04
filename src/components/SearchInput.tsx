import React, { useEffect, useRef } from 'react';
import { Search } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store';
import type { SearchResult } from '../types/ipc';
import './SearchInput.css';

const WEB_SEARCH_DEBOUNCE_MS = 600;

const WEB_RESULT_CATEGORIES = new Set(['Web Result', 'Web Answer']);

function asSearchResults(value: unknown): SearchResult[] {
  return Array.isArray(value) ? (value as SearchResult[]) : [];
}

const SearchInput: React.FC = () => {
  const { query, setQuery, setMode, setResults, mode } = useAppStore();
  const inputRef = useRef<HTMLInputElement>(null);
  const webSearchTimeoutRef = useRef<number | null>(null);

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

  const handleChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setQuery(val);

    if (val.startsWith('=')) {
      setMode('calculator');
      if (val.length > 1) {
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
          setResults([row]);
        } catch {
          /* invalid expression */
        }
      } else {
        setResults([]);
      }
      return;
    }

    if (val.startsWith('>')) {
      setMode('command');
      return;
    }

    if (val.startsWith('/')) {
      setMode('file');
      try {
        const queryStr = val === '/' ? '' : val.substring(1);
        const res = asSearchResults(
          await invoke<unknown>('search_files', { query: queryStr })
        );
        setResults(res);
      } catch (e) {
        console.error('File search error', e);
      }
      return;
    }

    if (mode === 'clipboard') {
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
        setResults(filtered);
      } catch (e) {
        console.error('Clipboard fetch error', e);
      }
      return;
    }

    setMode('default');
    if (val.trim() === '') {
      if (webSearchTimeoutRef.current !== null) {
        window.clearTimeout(webSearchTimeoutRef.current);
        webSearchTimeoutRef.current = null;
      }
      setResults([]);
      return;
    }

    if (val.length <= 2 && webSearchTimeoutRef.current !== null) {
      window.clearTimeout(webSearchTimeoutRef.current);
      webSearchTimeoutRef.current = null;
    }

    try {
      let res = asSearchResults(await invoke<unknown>('search', { query: val, category: null }));

      try {
        const calcRes = await invoke<string>('calculate', { expr: val });
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

      const webRow: SearchResult = {
        id: `web-search-${val}`,
        title: `Search DuckDuckGo for "${val}"`,
        subtitle: 'Opens in your default browser',
        category: 'Internet',
        icon: { kind: 'emoji', value: '🔍' },
        score: 0.1,
        actions: [{ id: 'search_web', title: 'Search DuckDuckGo', shortcut: '↵' }],
      };
      res = [...res, webRow];

      if (isUrl(val)) {
        const url = val.startsWith('http') ? val : `https://${val}`;
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

      setResults(res);

      if (val.length > 2) {
        const currentQuery = val;
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
              setResults([...nonWebResults, ...webResults]);
            }
          } catch (e) {
            console.error('Web fetch error', e);
          }
        }, WEB_SEARCH_DEBOUNCE_MS);
      }
    } catch (err) {
      console.error('Search error', err);
    }
  };

  return (
    <div className="search-input-container">
      <div className="search-icon">
        <Search size={16} />
      </div>
      {mode === 'file' && <span className="search-mode-pill mode-file">📂 Files</span>}
      {mode === 'calculator' && <span className="search-mode-pill mode-calc">🧮 Calc</span>}
      {mode === 'command' && <span className="search-mode-pill mode-cmd">⌘ CMD</span>}
      {mode === 'clipboard' && <span className="search-mode-pill mode-clip">📋 Clip</span>}
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={handleChange}
        onKeyDown={(e) => {
          if (e.key === 'Backspace' && query === '' && mode !== 'default') {
            setMode('default');
            setResults([]);
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
        spellCheck={false}
        autoComplete="off"
        autoCorrect="off"
      />
      <div className="search-badges">
        <kbd className="search-kbd-badge">⌘K</kbd>
      </div>
    </div>
  );
};

export default SearchInput;
