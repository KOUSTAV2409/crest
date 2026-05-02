import React from 'react';
import { useAppStore } from '../store';
import { motion } from 'framer-motion';
import './PreviewPanel.css';

const PreviewPanel: React.FC = () => {
  const { results, activeIndex } = useAppStore();
  const activeItem = results[activeIndex];

  if (!activeItem) return null;

  return (
    <motion.div
      key={activeItem.id}
      className="preview-panel fade-in"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.15 }}
    >
      <div className="preview-header">
        <div className="preview-icon-large">
          {activeItem.icon.kind === 'emoji' ? (
            <span className="emoji-icon-large">{activeItem.icon.value}</span>
          ) : (
            <div className="placeholder-icon-large" />
          )}
        </div>
        <h2 className="preview-title">{activeItem.title}</h2>
        <div className="preview-meta">
          <span>{activeItem.category}</span>
          {activeItem.subtitle && (
            <>
              <span className="meta-dot">•</span>
              <span>{activeItem.subtitle}</span>
            </>
          )}
        </div>
      </div>
      
      <div className="preview-body">
        {activeItem.preview?.description && (
          <p className="preview-description">{activeItem.preview.description}</p>
        )}
      </div>

      <div className="preview-actions">
        <div className="action-row">
          <span className="action-name">Open</span>
          <kbd className="action-kbd">↵</kbd>
        </div>
        <div className="action-row">
          <span className="action-name">Copy Info</span>
          <kbd className="action-kbd">⌘C</kbd>
        </div>
      </div>
    </motion.div>
  );
};

export default PreviewPanel;
