import React from 'react';
import { useAppStore } from '../store';
import { motion } from 'framer-motion';
import './PreviewPanel.css';

const getCategoryClass = (category: string) => {
  const c = category.toLowerCase();
  if (c === 'applications') return 'cat-app';
  if (c === 'files') return 'cat-file';
  if (c === 'calculator') return 'cat-calc';
  if (c === 'internet') return 'cat-web';
  return '';
};

const PreviewPanel: React.FC = () => {
  const { results, activeIndex } = useAppStore();
  const activeItem = results[activeIndex];

  if (!activeItem) return (
    <div className="preview-panel empty">
      <div className="empty-state">
        <div className="empty-icon">🏔️</div>
        <h3>Crest Launcher</h3>
        <p>Type to search apps, files, and more</p>
      </div>
    </div>
  );

  const isFile = activeItem.icon.kind === 'file';
  const isEmoji = activeItem.icon.kind === 'emoji';

  return (
    <motion.div
      key={activeItem.id}
      className="preview-panel"
      initial={{ opacity: 0, x: 5 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.15 }}
    >
      <div className="preview-content">
        {/* Large Hero Icon */}
        <div className="preview-hero">
          <div className={`hero-icon-container ${getCategoryClass(activeItem.category)}`}>
            {isEmoji ? (
              <span className="hero-emoji">{activeItem.icon.value}</span>
            ) : isFile ? (
              <span className="hero-file-type">{activeItem.icon.value.toUpperCase().slice(0, 3)}</span>
            ) : (
              <div className="hero-icon-app">
                <span className="hero-emoji">🚀</span>
              </div>
            )}
          </div>
          <h2 className="hero-title">{activeItem.title}</h2>
          <span className={`hero-badge ${getCategoryClass(activeItem.category)}`}>
            {activeItem.category}
          </span>
        </div>

        <div className="preview-divider" />

        {/* Metadata Section */}
        <div className="preview-metadata-list">
          {activeItem.subtitle && (
            <div className="metadata-row">
              <span className="metadata-label">{isFile ? 'Location' : 'Subtitle'}</span>
              <span className="metadata-value truncate">{activeItem.subtitle}</span>
            </div>
          )}
          
          {activeItem.preview?.description && (
            <div className="metadata-row vertical">
              <span className="metadata-label">Description</span>
              <p className="metadata-text">{activeItem.preview.description}</p>
            </div>
          )}

          {activeItem.category === 'Internet' && (
             <div className="metadata-row">
                <span className="metadata-label">Search</span>
                <span className="metadata-value">DuckDuckGo (browser + in-app)</span>
             </div>
          )}
        </div>
      </div>

      {/* Action Footer */}
      <div className="preview-footer">
        <div className="footer-action primary">
          <span className="action-key">↵</span>
          <span className="action-text">{activeItem.actions?.[0]?.title || 'Open'}</span>
        </div>
        <div className="footer-action secondary">
          <span className="action-key">⌘K</span>
          <span className="action-text">Actions</span>
        </div>
      </div>
    </motion.div>
  );
};

export default PreviewPanel;
