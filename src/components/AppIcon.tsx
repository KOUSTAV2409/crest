import React, { useEffect, useState } from 'react';
import './AppIcon.css';
import type { ResultIcon } from '../types/ipc';
import { resolveDesktopIconAssetUrl } from '../iconResolve';

type AppIconProps = {
  icon: ResultIcon;
  /** List row icons */
  variant?: 'row' | 'hero';
  className?: string;
};

/** Defer IPC so a wall of row mounts does not serialize behind the webview input path. */
function scheduleWhenIdle(run: () => void): () => void {
  if (typeof requestIdleCallback !== 'undefined') {
    const id = requestIdleCallback(run, { timeout: 200 });
    return () => cancelIdleCallback(id);
  }
  const id = window.setTimeout(run, 72);
  return () => clearTimeout(id);
}

export const AppIcon: React.FC<AppIconProps> = ({
  icon,
  variant = 'row',
  className = '',
}) => {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    if (icon.kind !== 'app' || !icon.value?.trim()) {
      setSrc(null);
      return undefined;
    }
    let cancelled = false;
    const cancelSchedule = scheduleWhenIdle(() => {
      if (cancelled) return;
      void resolveDesktopIconAssetUrl(icon.value).then((url) => {
        if (!cancelled) setSrc(url);
      });
    });
    return () => {
      cancelled = true;
      cancelSchedule();
    };
  }, [icon.kind, icon.value]);

  const rowClass = variant === 'row' ? 'app-icon app-icon--row' : 'app-icon app-icon--hero';

  if (icon.kind === 'emoji') {
    return (
      <span className={`${rowClass} app-icon--emoji ${className}`.trim()} aria-hidden>
        {icon.value}
      </span>
    );
  }

  if (icon.kind === 'file') {
    return (
      <span className={`${rowClass} app-icon-file-ext ${className}`.trim()}>
        {icon.value ? icon.value.toUpperCase().slice(0, 3) : '📄'}
      </span>
    );
  }

  if (src) {
    return (
      <img
        src={src}
        alt=""
        draggable={false}
        className={`${rowClass} app-icon-img ${className}`.trim()}
        onError={() => setSrc(null)}
      />
    );
  }

  return (
    <span className={`${rowClass} app-icon-fallback ${className}`.trim()} aria-hidden>
      ★
    </span>
  );
};
