import { useEffect } from 'react';
import CommandPalette from './components/CommandPalette';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import UpdateNotification from './components/UpdateNotification';
import './index.css';

function App() {
  useEffect(() => {
    // Disable right click menu
    const handleContextMenu = (e: Event) => e.preventDefault();
    document.addEventListener('contextmenu', handleContextMenu);

    // Global key listeners
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        getCurrentWindow().hide();
      }
      if (e.ctrlKey && e.key === 'q') {
        invoke('quit_app');
      }
    };
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('contextmenu', handleContextMenu);
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  return (
    <div className="window-container">
      <CommandPalette />
      <UpdateNotification />
    </div>
  );
}

export default App;
