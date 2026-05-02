import React, { useRef, useEffect } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { useAppStore } from '../store';
import clsx from 'clsx';
import { ChevronRight } from 'lucide-react';
import { motion } from 'framer-motion';
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
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [activeIndex, results.length, setActiveIndex, rowVirtualizer]);

  if (results.length === 0) {
    return (
      <div className="empty-state">
        <p>No results found</p>
      </div>
    );
  }

  return (
    <div ref={parentRef} className="result-list-scroll">
      <div
        className="virtual-list-inner"
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,
        }}
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
              }}
              onMouseEnter={() => setActiveIndex(virtualRow.index)}
            >
              {isActive && (
                <motion.div 
                  layoutId="active-selection" 
                  className="active-bg"
                  initial={false}
                  transition={{ type: "spring", stiffness: 500, damping: 30 }}
                />
              )}
              <div className="item-content">
                <div className="item-icon">
                  {item.icon.kind === 'emoji' ? (
                    <span className="emoji-icon">{item.icon.value}</span>
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
