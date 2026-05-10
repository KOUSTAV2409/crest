import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { check, Update } from '@tauri-apps/plugin-updater';
import './UpdateNotification.css';

const UpdateNotification: React.FC = () => {
  const [update, setUpdate] = useState<Update | null>(null);
  const [isInstalling, setIsInstalling] = useState(false);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    // Check for updates shortly after app start
    const timer = setTimeout(async () => {
      try {
        const result = await check();
        if (result?.available) {
          setUpdate(result);
          setIsVisible(true);
        }
      } catch (error) {
        console.error('Failed to check for updates:', error);
      }
    }, 2000);

    return () => clearTimeout(timer);
  }, []);

  const handleInstall = async () => {
    if (!update) return;
    setIsInstalling(true);
    try {
      await update.downloadAndInstall();
      // App should restart automatically after install, but just in case:
      setIsVisible(false);
    } catch (error) {
      console.error('Failed to install update:', error);
      setIsInstalling(false);
    }
  };

  const handleDismiss = () => {
    setIsVisible(false);
  };

  return (
    <AnimatePresence>
      {isVisible && update && (
        <div className="update-notification-overlay">
          <motion.div
            className="update-notification-modal"
            initial={{ opacity: 0, y: 20, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 10, scale: 0.95 }}
            transition={{ type: "spring", bounce: 0, duration: 0.4 }}
          >
            <button className="update-close-btn" onClick={handleDismiss} aria-label="Close">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M11 1L1 11M1 1L11 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>
            
            <h2 className="update-title">Crest v{update.version} is available</h2>
            <p className="update-body">
              See the assets to download and install. Auto-update is built in.
            </p>
            
            <div className="update-actions">
              <button className="update-btn-later" onClick={handleDismiss}>
                Later
              </button>
              <button 
                className="update-btn-install" 
                onClick={handleInstall}
                disabled={isInstalling}
              >
                {isInstalling ? (
                  <>
                    <svg className="animate-spin" width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                      <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="3" strokeOpacity="0.2"/>
                      <path d="M12 2C6.47715 2 2 6.47715 2 12" stroke="currentColor" strokeWidth="3" strokeLinecap="round"/>
                    </svg>
                    Installing...
                  </>
                ) : (
                  'Install & restart'
                )}
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
};

export default UpdateNotification;
