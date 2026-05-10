import React, { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '../store';
import SearchInput from './SearchInput';
import ResultList from './ResultList';
import PreviewPanel from './PreviewPanel';
import TerminalPanel from './TerminalPanel';
import ShortcutSetupBanner from './ShortcutSetupBanner';
import clsx from 'clsx';
import './CommandPalette.css';

const modeLabel: Record<string, string> = {
  default: 'Applications',
  file: 'Files',
  calculator: 'Calculator',
  command: 'Commands',
  clipboard: 'Clipboard',
};

const CommandPalette: React.FC = () => {
  const results = useAppStore((s) => s.results);
  const mode = useAppStore((s) => s.mode);
  const activeTerminalCommand = useAppStore((s) => s.activeTerminalCommand);

  useEffect(() => {
    // Keyboard navigation handled in ResultList
  }, []);

  return (
    <AnimatePresence>
      <motion.div
        className="command-palette fade-in"
        initial={{ scale: 0.97, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.97, opacity: 0 }}
        transition={{ duration: 0.1, ease: [0.16, 1, 0.3, 1] }}
      >
        <SearchInput />
        <div
          className="palette-drag-rail"
          data-tauri-drag-region
          aria-hidden
          title="Drag to move window"
        />
        <ShortcutSetupBanner />

        <div className="palette-body">
          <div className={clsx('result-list-container', { 'full-width': results.length === 0 })}>
            <ResultList />
          </div>

          {results.length > 0 && (
            <div className="preview-panel-container">
              {activeTerminalCommand !== null ? (
                <TerminalPanel command={activeTerminalCommand} />
              ) : (
                <PreviewPanel />
              )}
            </div>
          )}
        </div>

        <div className="status-bar" data-tauri-drag-region>
          <div className="status-left">
            <div className="status-category-chip">
              {modeLabel[mode] ?? mode}
            </div>
            <div className="status-divider" />
            <div className="status-action-hint">
              <span className="action-text">Actions</span>
              <span className="status-kbd-group">
                <kbd className="status-kbd">Ctrl</kbd>
                <kbd className="status-kbd">K</kbd>
              </span>
            </div>
          </div>
          <div className="status-right">
            <div className="status-shortcut">
              <span className="status-count">{results.length} results</span>
            </div>
            <div className="status-shortcut">
              <kbd className="status-kbd">↵</kbd>
              <span>Open</span>
            </div>
          </div>
        </div>
      </motion.div>
    </AnimatePresence>
  );
};

export default CommandPalette;
