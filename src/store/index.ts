import { create } from 'zustand';

export interface SearchResult {
  id: string;
  title: string;
  subtitle: string;
  icon: { kind: string; value: string };
  category: string;
  score: number;
  actions: any[];
  preview?: any;
}

interface AppState {
  query: string;
  setQuery: (q: string) => void;
  results: SearchResult[];
  setResults: (r: SearchResult[]) => void;
  activeIndex: number;
  setActiveIndex: (index: number) => void;
  activeCategory: string | null;
  setActiveCategory: (cat: string | null) => void;
  mode: 'default' | 'calculator' | 'file' | 'command';
  setMode: (mode: 'default' | 'calculator' | 'file' | 'command') => void;
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
  setMode: (m) => set({ mode: m })
}));
