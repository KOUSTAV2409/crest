/** Matches `commands::search` IPC payloads (camelCase rename on the Rust side). */

export type ResultIconKind = 'app' | 'emoji' | 'file' | 'url' | string;

export interface ResultIcon {
  kind: ResultIconKind;
  value: string;
}

export interface Action {
  id: string;
  title: string;
  shortcut?: string;
}

export interface Preview {
  title: string;
  subtitle?: string;
  description?: string;
}

export interface SearchResult {
  id: string;
  title: string;
  subtitle: string;
  icon: ResultIcon;
  category: string;
  score: number;
  actions: Action[];
  preview?: Preview | null;
}

export interface SystemAction {
  id: string;
  title: string;
  icon: string;
}
