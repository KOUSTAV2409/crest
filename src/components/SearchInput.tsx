import React, { useEffect, useState, useRef } from 'react';
import { Search } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store';
import './SearchInput.css';

const SearchInput: React.FC = () => {
  const { query, setQuery, setMode, setResults, mode } = useAppStore();
  const inputRef = useRef<HTMLInputElement>(null);
  
  const [placeholder, setPlaceholder] = useState("Search apps...");
  
  useEffect(() => {
    // Focus input on mount
    inputRef.current?.focus();
    
    // Cycle placeholder
    const placeholders = ["Search apps...", "Run commands...", "Find files..."];
    let i = 0;
    const interval = setInterval(() => {
      if (!query) {
        i = (i + 1) % placeholders.length;
        setPlaceholder(placeholders[i]);
      }
    }, 3000);
    return () => clearInterval(interval);
  }, [query]);

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
            actions: []
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
    } else {
      setMode('default');
      if (val.trim() === '') {
        setResults([]);
        return;
      }
      try {
        const res: any = await invoke('search', { query: val, category: null });
        
        // Try calculator implicitly
        try {
           const calcRes = await invoke('calculate', { expr: val });
           let calcResStr = calcRes as string;
           // Format long decimals to 2 decimal places for cleaner UI (e.g. 95.041538 -> 95.04)
           calcResStr = calcResStr.replace(/(\.\d{2})\d+/, '$1');
           
           res.unshift({
              id: 'calc',
              title: calcResStr,
              subtitle: 'Result',
              category: 'Calculator',
              icon: { kind: 'emoji', value: '🧮' },
              score: 1,
              actions: []
           });
        } catch(e) {}
        
        setResults(res);
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
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={handleChange}
        placeholder={placeholder}
        className="search-input"
        spellCheck={false}
        autoComplete="off"
        autoCorrect="off"
      />
      <div className="search-badges">
        <kbd className="kbd-badge">⌘K</kbd>
      </div>
    </div>
  );
};

export default SearchInput;
