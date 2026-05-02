import { useEffect } from 'react';
import CommandPalette from './components/CommandPalette';
import './index.css';

function App() {
  useEffect(() => {
    // Disable right click menu
    const handleContextMenu = (e: Event) => e.preventDefault();
    document.addEventListener('contextmenu', handleContextMenu);
    return () => document.removeEventListener('contextmenu', handleContextMenu);
  }, []);

  return (
    <div className="window-container">
      <CommandPalette />
    </div>
  );
}

export default App;
