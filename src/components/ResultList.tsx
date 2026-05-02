import React from 'react';
import { useAppStore } from '../store';
import { useVirtualizer } from '@tanstack/react-virtual';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import { ChevronRight } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import './ResultList.css';

const ITEM_HEIGHT = 44;

const ResultList: React.FC = () => {
  const { results, activeIndex, setActiveIndex, setIndex } = useAppStore();
  const parentRef = React.useRef<HTMLDivElement>(null);

  const rowVirtualizer = useVirtualizer({
    count: results.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ITEM_HEIGHT,
    overscan: 10,
  });

  const activateItem = async (index: number) => {
    const item = results[index];
    if (!item || !item.actions || item.actions.length === 0) return;

    const action = item.actions[0];
    const actionId = action.id;

    console.log('Activating action:', actionId, 'for item:', item.id);

    if (actionId === 'launch') {
      invoke('launch_app', { appId: item.id })
        .then(() => console.log('Launched app:', item.title))
        .catch((e) => console.error('launch_app error:', e));
    } else if (actionId === 'search_web') {
      const query = item.id.replace('web-search-', '');
      invoke('search_web', { query })
        .then(() => console.log('Web search for:', query))
        .catch((e) => console.error('search_web error:', e));
    } else if (actionId === 'open_url') {
      let url = item.id;
      if (url.startsWith('open-url-')) url = url.replace('open-url-', '');
      if (url.startsWith('web-abs-')) url = url.replace('web-abs-', '');
      if (url.startsWith('web-rel-')) url = url.replace('web-rel-', '');
      if (url.startsWith('web-lite-')) url = url.replace('web-lite-', '');
      
      invoke('open_file', { path: url }) 
        .then(() => console.log('Opened URL:', url))
        .catch((e) => console.error('open_url error:', e));
    } else if (actionId === 'copy') {
      try {
        await navigator.clipboard.writeText(item.title);
        console.log('Copied to clipboard:', item.title);
      } catch (e) {
        console.error('Clipboard error:', e);
      }
    } else if (actionId === 'open_file') {
      invoke('open_file', { path: item.subtitle })
        .then(() => console.log('Opened file:', item.subtitle))
        .catch((e) => console.error('open_file error:', e));
    }
  };

  // Sync scroll position when activeIndex changes
  React.useEffect(() => {
    if (activeIndex !== -1) {
      rowVirtualizer.scrollToIndex(activeIndex, { align: 'center' });
    }
  }, [activeIndex, rowVirtualizer]);

  if (results.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">🔍</div>
        <p>No results found</p>
        <span className="empty-state-hint">Try a different search term</span>
      </div>
    );
  }

  return (
    <div className="result-list-scroll" ref={parentRef}>
      <div
        className="virtual-list-inner"
        style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
      >
        {rowVirtualizer.getVirtualItems().map((virtualRow) => {
          const item = results[virtualRow.index];
          const isActive = activeIndex === virtualRow.index;
          return (
            <div
              key={item.id}
              className={clsx('result-item', { active: isActive })}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start}px)`,
                cursor: 'pointer',
              }}
              onMouseEnter={() => setActiveIndex(virtualRow.index)}
              onDoubleClick={() => activateItem(virtualRow.index)}
              onClick={() => {
                if (isActive) {
                  activateItem(virtualRow.index);
                } else {
                  setActiveIndex(virtualRow.index);
                }
              }}
            >
              {isActive && (
                <motion.div
                  layoutId="active-selection"
                  className="active-bg"
                  initial={false}
                  transition={{ type: 'spring', stiffness: 500, damping: 30 }}
                />
              )}
              <div className="item-content">
                <div className="item-icon">
                  {item.icon.kind === 'emoji' ? (
                    <span className="emoji-icon">{item.icon.value}</span>
                  ) : item.icon.kind === 'file' ? (
                    <span className="file-ext-badge">
                      {item.icon.value ? item.icon.value.toUpperCase().slice(0, 3) : '📄'}
                    </span>
                  ) : (
                    <div className="placeholder-icon">🚀</div>
                  )}
                </div>
                
                <div className="item-text">
                  <span className="item-title">{item.title}</span>
                  {item.subtitle && <span className="item-subtitle truncate">{item.subtitle}</span>}
                </div>

                <div className="item-actions">
                   {!isActive && <span className="item-category-label">{item.category}</span>}
                   {isActive && item.actions?.[0]?.shortcut && (
                     <span className="item-shortcut">{item.actions[0].shortcut}</span>
                   )}
                   {isActive && <ChevronRight size={14} className="chevron" />}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default ResultList;
