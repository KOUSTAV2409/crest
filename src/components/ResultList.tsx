import React from 'react';
import { useAppStore } from '../store';
import { useVirtualizer } from '@tanstack/react-virtual';
import clsx from 'clsx';
import { ChevronRight } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import type { SearchResult } from '../types/ipc';
import { categorySlug } from '../categorySlug';
import { AppIcon } from './AppIcon';
import './ResultList.css';

const ITEM_HEIGHT = 62; // Must match .result-item height in CSS

const ResultList: React.FC = () => {
  const results = useAppStore((s) => s.results);
  const activeIndex = useAppStore((s) => s.activeIndex);
  const setActiveIndex = useAppStore((s) => s.setActiveIndex);
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
      
      invoke('open_url', { url }) 
        .then(() => console.log('Opened URL:', url))
        .catch((e) => console.error('open_url error:', e));
    } else if (actionId === 'quit') {
      invoke('quit_app');
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
    } else if (actionId === 'open_clipboard') {
      const { setMode, setQuery } = useAppStore.getState();
      setMode('clipboard');
      setQuery(''); // Clear query to show all history
    } else if (actionId === 'run_extension') {
      invoke<unknown>('run_extension', { id: item.id, action: 'run', args: {} })
        .then((res) => {
          console.log('Extension results:', res);
          if (Array.isArray(res)) {
            const { setResults } = useAppStore.getState();
            setResults(res as SearchResult[]);
          }
        })
        .catch((e) => console.error('run_extension error:', e));
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
        <div className="empty-state-content">
          <div className="empty-state-icon">🔍</div>
          <h3 className="empty-state-title">No results found</h3>
          <p className="empty-state-description">Try a different search term or run a command.</p>
        </div>
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
          const shortcut = item.actions?.[0]?.shortcut ?? null;
          return (
            <div
              key={item.id}
              className={clsx('result-item', { active: isActive })}
              data-category={categorySlug(item.category)}
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
              {isActive && <div className="active-bg" />}
              <div className="item-content">
                <span className="category-marker" aria-hidden />
                <div className="item-icon">
                  <AppIcon icon={item.icon} variant="row" />
                </div>

                <div className="item-text">
                  <span className="item-title">{item.title}</span>
                  {item.subtitle && <span className="item-subtitle truncate">{item.subtitle}</span>}
                  {item.preview?.description && (
                    <span className="item-description-snippet">
                      {item.preview.description}
                    </span>
                  )}
                </div>

                <div className="item-actions">
                  {shortcut && !isActive && (
                    <span className="item-shortcut">{shortcut}</span>
                  )}
                  {shortcut && isActive && (
                    <span className="item-shortcut item-shortcut--active">Enter {shortcut}</span>
                  )}
                  {isActive && <ChevronRight size={15} className="chevron" strokeWidth={2.25} />}
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
