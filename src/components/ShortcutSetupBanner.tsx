import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './ShortcutSetupBanner.css';

const DISMISS_KEY = 'crest_shortcut_setup_banner_dismissed';

interface ShortcutSetupHint {
  show_banner: boolean;
  headline: string;
  detail: string;
}

const ShortcutSetupBanner: React.FC = () => {
  const [hint, setHint] = useState<ShortcutSetupHint | null>(null);
  const [dismissed, setDismissed] = useState(
    () => localStorage.getItem(DISMISS_KEY) === '1'
  );

  useEffect(() => {
    invoke<ShortcutSetupHint>('get_shortcut_setup_hint')
      .then((h) => {
        if (h.show_banner && h.headline) setHint(h);
      })
      .catch(() => {
        /* non-Tauri env */
      });
  }, []);

  if (dismissed || !hint?.show_banner) return null;

  return (
    <div className="shortcut-setup-banner" role="note">
      <div className="shortcut-setup-banner-inner">
        <div className="shortcut-setup-banner-text">
          <span className="shortcut-setup-banner-title">{hint.headline}</span>
          <p className="shortcut-setup-banner-detail">{hint.detail}</p>
        </div>
        <button
          type="button"
          className="shortcut-setup-banner-dismiss"
          onClick={() => {
            localStorage.setItem(DISMISS_KEY, '1');
            setDismissed(true);
          }}
        >
          OK
        </button>
      </div>
    </div>
  );
};

export default ShortcutSetupBanner;
