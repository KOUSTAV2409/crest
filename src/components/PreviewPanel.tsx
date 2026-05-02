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

  if (!activeItem) return null;

  const isFile = activeItem.icon.kind === 'file';
  const isEmoji = activeItem.icon.kind === 'emoji';

  return (
    <motion.div
      key={activeItem.id}
      className="preview-panel fade-in"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.12 }}
    >
      {/* Header */}
      <div className="preview-header">
        <div className="preview-icon-wrap">
          {isEmoji ? (
            <span className="emoji-icon-large">{activeItem.icon.value}</span>
          ) : isFile ? (
            <span className="preview-file-badge">
              {activeItem.icon.value ? activeItem.icon.value.toUpperCase().slice(0, 4) : 'FILE'}
            </span>
          ) : (
            <div className="placeholder-icon-large" />
          )}
        </div>

        <h2 className="preview-title">{activeItem.title}</h2>

        <div className="preview-meta">
          <span className={`meta-chip ${getCategoryClass(activeItem.category)}`}>
            {activeItem.category}
          </span>
          {activeItem.subtitle && activeItem.category !== 'Files' && (
            <>
              <div className="meta-dot" />
              <span className="meta-path">{activeItem.subtitle}</span>
            </>
          )}
        </div>
      </div>

      {/* Body */}
      <div className="preview-body">
        {activeItem.preview?.description && (
          <p className="preview-description">{activeItem.preview.description}</p>
        )}
        {isFile && activeItem.subtitle && (
          <div className="preview-path-full">{activeItem.subtitle}</div>
        )}
      </div>

      {/* Actions */}
      <div className="preview-actions">
        <div className="action-row">
          <span className="action-name">Open</span>
          <kbd className="action-kbd">↵</kbd>
        </div>
        <div className="action-row">
          <span className="action-name">Copy Info</span>
          <kbd className="action-kbd">⌘G</kbd>
        </div>
      </div>
    </motion.div>
  );
};

export default PreviewPanel;
