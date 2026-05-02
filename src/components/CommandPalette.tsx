import React, { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '../store';
import SearchInput from './SearchInput';
import ResultList from './ResultList';
import PreviewPanel from './PreviewPanel';
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
  const { results, mode } = useAppStore();

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

        <div className="palette-body">
          <div className={clsx('result-list-container', { 'full-width': results.length === 0 })}>
            <ResultList />
          </div>

          {results.length > 0 && (
            <div className="preview-panel-container">
              <PreviewPanel />
            </div>
          )}
        </div>

        {/* Status bar */}
        <div className="status-bar">
          <div className="status-left">
            <div className="status-category-chip">
              <div className="status-dot" />
              {modeLabel[mode] ?? mode}
            </div>
          </div>
          <div className="status-right">
            <span className="status-count">{results.length} results</span>
            <div className="status-shortcut">
              <kbd className="status-kbd">↑↓</kbd>
              <span>Navigate</span>
            </div>
            <div className="status-shortcut">
              <kbd className="status-kbd">↵</kbd>
              <span>Open</span>
            </div>
            <div className="status-shortcut">
              <kbd className="status-kbd">Esc</kbd>
              <span>Close</span>
            </div>
          </div>
        </div>
      </motion.div>
    </AnimatePresence>
  );
};

export default CommandPalette;
