import React, { useRef, useEffect, useCallback } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { useAppStore } from '../store';
import clsx from 'clsx';
import { ChevronRight } from 'lucide-react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import './ResultList.css';

const ResultList: React.FC = () => {
  const { results, activeIndex, setActiveIndex } = useAppStore();
  const parentRef = useRef<HTMLDivElement>(null);

  const rowVirtualizer = useVirtualizer({
    count: results.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 52,
    overscan: 5,
  });

  // Shared activation logic — used by keyboard Enter and mouse click/double-click
  const activateItem = useCallback((index: number) => {
    const item = results[index];
    if (!item) return;

    if (item.actions.length === 0) {
      console.warn('No actions for item:', item.title);
      return;
    }

    const actionId = item.actions[0].id;
    console.log('Activating:', item.title, '| action:', actionId, '| id:', item.id);

    if (actionId === 'launch') {
      invoke('launch_app', { appId: item.id })
        .then(() => console.log('Launched:', item.title))
        .catch((e) => console.error('launch_app error:', e));
    } else if (actionId === 'open_file') {
      invoke('open_file', { path: item.id })
        .then(() => console.log('Opened file:', item.id))
        .catch((e) => console.error('open_file error:', e));
    }
  }, [results]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (results.length === 0) return;
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        const next = Math.min(activeIndex + 1, results.length - 1);
        setActiveIndex(next);
        rowVirtualizer.scrollToIndex(next);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        const prev = Math.max(activeIndex - 1, 0);
        setActiveIndex(prev);
        rowVirtualizer.scrollToIndex(prev);
      } else if (e.key === 'Enter') {
        e.preventDefault();
        activateItem(activeIndex);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [activeIndex, results.length, setActiveIndex, rowVirtualizer, activateItem]);

  if (results.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">⌘</div>
        <p>No results found</p>
        <span className="empty-state-hint">Try a different search, or type / for files</span>
      </div>
    );
  }

  return (
    <div ref={parentRef} className="result-list-scroll">
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
              // Double-click always opens immediately
              onDoubleClick={() => activateItem(virtualRow.index)}
              // Single click: first click selects, second click on same item opens
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
                      {item.icon.value ? item.icon.value.toUpperCase().slice(0, 4) : '📄'}
                    </span>
                  ) : (
                    <div className="placeholder-icon" />
                  )}
                </div>
                <div className="item-text">
                  <span className="item-title">{item.title}</span>
                  {item.subtitle && <span className="item-subtitle">{item.subtitle}</span>}
                </div>
                <div className="item-actions">
                  {isActive && <ChevronRight size={16} className="chevron" />}
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
