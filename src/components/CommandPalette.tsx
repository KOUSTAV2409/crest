import React, { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '../store';
import SearchInput from './SearchInput';
import ResultList from './ResultList';
import PreviewPanel from './PreviewPanel';
import clsx from 'clsx';
import './CommandPalette.css';

const CommandPalette: React.FC = () => {
  const { results, mode } = useAppStore();

  useEffect(() => {
    // Keyboard navigation handled here or in a hook
  }, []);

  return (
    <AnimatePresence>
      <motion.div
        className="command-palette fade-in"
        initial={{ scale: 0.95, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.95, opacity: 0 }}
        transition={{ duration: 0.12, ease: [0.16, 1, 0.3, 1] }}
      >
        <SearchInput />
        
        <div className="palette-body">
          <div className={clsx("result-list-container", { "full-width": results.length === 0 })}>
            <ResultList />
          </div>
          
          {results.length > 0 && (
            <div className="preview-panel-container">
              <PreviewPanel />
            </div>
          )}
        </div>
        
        <div className="status-bar">
          <div className="status-left">
            <span>{mode === 'default' ? 'Applications' : mode}</span>
          </div>
          <div className="status-right">
            <span>{results.length} results</span>
            <span className="status-shortcuts">↑↓ Navigate</span>
          </div>
        </div>
      </motion.div>
    </AnimatePresence>
  );
};

export default CommandPalette;
