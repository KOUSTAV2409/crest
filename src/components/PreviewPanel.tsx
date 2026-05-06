import React from 'react';
import { useAppStore } from '../store';
import { AppIcon } from './AppIcon';
import './PreviewPanel.css';

const getCategoryClass = (category: string) => {
  const c = category.toLowerCase();
  if (c === 'applications') return 'cat-app';
  if (c === 'files' || c === 'file') return 'cat-file';
  if (c === 'calculator') return 'cat-calc';
  if (c === 'internet' || c.includes('web')) return 'cat-web';
  if (c === 'system') return 'cat-system';
  if (c === 'extension' || c === 'extensions') return 'cat-extension';
  if (c === 'commands') return 'cat-cmds';
  if (c === 'clipboard') return 'cat-clipboard';
  return '';
};

function resultSourceLabel(category: string): string {
  const c = category.toLowerCase();
  if (c === 'internet' || c.includes('web')) return 'Web';
  return 'Local';
}

const PreviewPanel: React.FC = () => {
  const results = useAppStore((s) => s.results);
  const activeIndex = useAppStore((s) => s.activeIndex);
  const activeItem = results[activeIndex];

  if (!activeItem) {
    return (
      <div className="preview-panel empty">
        <div className="empty-state">
          <div className="empty-icon">🏔️</div>
          <h3>Crest</h3>
          <p>Search apps, files, and commands</p>
        </div>
      </div>
    );
  }

  const catClass = getCategoryClass(activeItem.category);
  const source = (() => {
    // Some rows use "Internet" category for an action that opens browser; treat those as Web.
    const c = activeItem.category.toLowerCase();
    if (c === 'internet' || c.includes('web')) return 'Web';
    if (
      activeItem.id.startsWith('web-') ||
      activeItem.id.startsWith('open-url-') ||
      activeItem.id.startsWith('open-url') ||
      activeItem.id.startsWith('wiki-')
    ) {
      return 'Web';
    }
    return resultSourceLabel(activeItem.category);
  })();

  return (
    <div key={activeItem.id} className="preview-panel preview-panel--enter">
      <div className="preview-content">
        <div className="preview-hero">
          <div className="hero-icon-wrapper">
            <AppIcon icon={activeItem.icon} variant="hero" />
          </div>
          <div className="hero-text-stack">
            <h2 className="hero-title">{activeItem.title}</h2>
            {activeItem.subtitle && (
              <p className="hero-subtitle">{activeItem.subtitle}</p>
            )}
          </div>
        </div>

        <div className="preview-divider" />

        <div className="preview-body">
          {activeItem.preview?.description && (
            <p className="preview-description-text">
              {activeItem.preview.description}
            </p>
          )}

          <div className="preview-metadata-block">
            <div className="meta-line">
              <span className="meta-label">Category</span>
              <span className={`meta-value ${catClass}`}>
                {activeItem.category === 'Applications' ? 'App' : activeItem.category}
              </span>
            </div>
            <div className="meta-line">
              <span className="meta-label">Source</span>
            <span
              className={
                source === 'Local'
                  ? 'meta-value accent-amber'
                  : 'meta-value meta-value--muted'
              }
            >
              {source}
            </span>
            </div>
          </div>
        </div>

        <div className="preview-hint-box">
          <div className="hint-stack">
            <div className="hint-line">
              <span className="hint-prompt">›_</span>
              <span>Type to filter results</span>
            </div>
            <div className="hint-line">
              <span className="hint-icon">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
                </svg>
              </span>
              <span>Use arrows to navigate</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PreviewPanel;
