import React, { useEffect, useState, useRef } from 'react';
import { Search } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store';
import './SearchInput.css';

const SearchInput: React.FC = () => {
  const { query, setQuery, setMode, setResults, mode } = useAppStore();
  const inputRef = useRef<HTMLInputElement>(null);
  
  const modeLabel: Record<string, string> = {
    default: 'apps and commands',
    calculator: 'calculator',
    command: 'commands',
    file: 'files',
    clipboard: 'clipboard history'
  };
  
  useEffect(() => {
    // Focus input on mount
    inputRef.current?.focus();
  }, []);

  const currentPlaceholder = mode === 'default' 
    ? 'Search for apps and commands...' 
    : `Search in ${modeLabel[mode]}...`;

  const isUrl = (str: string) => {
    try {
      const url = str.trim();
      return url.startsWith('http://') || url.startsWith('https://') || 
             (/^([a-z0-9]+(-[a-z0-9]+)*\.)+[a-z]{2,}(:\d+)?(\/.*)?$/i.test(url));
    } catch { return false; }
  };

  const handleChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setQuery(val);
    
    if (val.startsWith('=')) {
      setMode('calculator');
      if (val.length > 1) {
        try {
          const res = await invoke('calculate', { expr: val.substring(1) });
          setResults([{
            id: 'calc',
            title: res as string,
            subtitle: 'Result',
            category: 'Calculator',
            icon: { kind: 'emoji', value: '🧮' },
            score: 1,
            actions: [{ id: 'copy', title: 'Copy Result', shortcut: '↵' }]
          }]);
        } catch (e) {
          // Ignore parse errors
        }
      } else {
        setResults([]);
      }
    } else if (val.startsWith('>')) {
      setMode('command');
      // Implement command search invoke
    } else if (val.startsWith('/')) {
      setMode('file');
      try {
        const queryStr = val === '/' ? '' : val.substring(1);
        const res: any = await invoke('search_files', { query: queryStr });
        setResults(res);
      } catch (e) {
        console.error("File search error", e);
      }
    } else if (mode === 'clipboard') {
        try {
          const res: any = await invoke('get_clipboard_history');
          // Filter locally for now
          const filtered = res.filter((item: any) => 
            item.title.toLowerCase().includes(val.toLowerCase()) || 
            (item.preview?.description && item.preview.description.toLowerCase().includes(val.toLowerCase()))
          );
          setResults(filtered);
        } catch (e) {
          console.error("Clipboard fetch error", e);
        }
    } else {
      setMode('default');
      if (val.trim() === '') {
        setResults([]);
        return;
      }
      try {
        let res: any = await invoke('search', { query: val, category: null });
        
        // 1. Try calculator implicitly
        try {
           const calcRes = await invoke('calculate', { expr: val });
           let calcResStr = calcRes as string;
           calcResStr = calcResStr.replace(/(\.\d{2})\d+/, '$1');
           
           res.unshift({
              id: 'calc',
              title: calcResStr,
              subtitle: 'Calculator Result',
              category: 'Calculator',
              icon: { kind: 'emoji', value: '🧮' },
              score: 1.1,
              actions: [{ id: 'copy', title: 'Copy Result', shortcut: '↵' }]
           });
        } catch(e) {}

        // 2. Add "Search Google" option
        res.push({
          id: `web-search-${val}`,
          title: `Search Google for "${val}"`,
          subtitle: 'Web Search',
          category: 'Internet',
          icon: { kind: 'emoji', value: '🔍' },
          score: 0.1,
          actions: [{ id: 'search_web', title: 'Search Google', shortcut: '↵' }]
        });

        // 3. Check if it looks like a URL
        if (isUrl(val)) {
          const url = val.startsWith('http') ? val : `https://${val}`;
          res.unshift({
            id: `open-url-${url}`,
            title: `Open ${url}`,
            subtitle: 'Web Browser',
            category: 'Internet',
            icon: { kind: 'emoji', value: '🌐' },
            score: 1.2,
            actions: [{ id: 'open_url', title: 'Open in Browser', shortcut: '↵' }]
          });
        }
        
        setResults(res);

        // 4. In-app Web Search (Background Fetch)
        if (val.length > 2) { // Lowered to 2 for faster feedback
            const currentQuery = val;
            clearTimeout((window as any).webSearchTimeout);
            (window as any).webSearchTimeout = setTimeout(async () => {
                console.log("Triggering web search for:", currentQuery);
                try {
                    const webResults: any = await invoke('fetch_web_results', { query: currentQuery });
                    console.log("Received web results:", webResults.length);
                    
                    // Get latest query to prevent race conditions
                    const latestQuery = (useAppStore.getState() as any).query;
                    
                    if (webResults.length > 0 && latestQuery === currentQuery) {
                        console.log("Injecting web results into UI");
                        // Merge with the LATEST results from the store, not the stale 'res'
                        const currentResults = (useAppStore.getState() as any).results;
                        // Filter out existing web results if any (to prevent duplicates)
                        const nonWebResults = currentResults.filter((r: any) => r.category !== 'Web Result' && r.category !== 'Web Answer');
                        setResults([...nonWebResults, ...webResults]);
                    }
                } catch (e) {
                    console.error("Web fetch error", e);
                }
            }, 600);
        }

      } catch (err) {
        console.error("Search error", err);
      }
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
              // Trigger click on the active item
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
