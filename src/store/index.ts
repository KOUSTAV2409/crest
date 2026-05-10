import { create } from 'zustand';
import type { SearchResult } from '../types/ipc';

interface AppState {
  query: string;
  setQuery: (q: string) => void;
  results: SearchResult[];
  setResults: (r: SearchResult[]) => void;
  activeIndex: number;
  setActiveIndex: (index: number) => void;
  activeCategory: string | null;
  setActiveCategory: (cat: string | null) => void;
  mode: 'default' | 'calculator' | 'file' | 'command' | 'clipboard' | 'terminal';
  setMode: (mode: 'default' | 'calculator' | 'file' | 'command' | 'clipboard' | 'terminal') => void;
  activeTerminalCommand: string | null;
  setActiveTerminalCommand: (cmd: string | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  query: '',
  setQuery: (q) => set({ query: q }),
  results: [],
  setResults: (r) => set({ results: r, activeIndex: 0 }),
  activeIndex: 0,
  setActiveIndex: (i) => set({ activeIndex: i }),
  activeCategory: null,
  setActiveCategory: (c) => set({ activeCategory: c }),
  mode: 'default',
  setMode: (m) => set({ mode: m, activeTerminalCommand: null }),
  activeTerminalCommand: null,
  setActiveTerminalCommand: (cmd) => set({ activeTerminalCommand: cmd }),
}));
