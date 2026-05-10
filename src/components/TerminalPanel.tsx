import React, { useEffect, useRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import './TerminalPanel.css';

interface TerminalPanelProps {
  command: string;
}

const TerminalPanel: React.FC<TerminalPanelProps> = ({ command }) => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const term = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const isMounted = useRef(false);

  useEffect(() => {
    if (!terminalRef.current || isMounted.current) return;
    isMounted.current = true;

    term.current = new Terminal({
      theme: {
        background: '#12121e',
        foreground: '#ffffff',
        cursor: '#7c4dff',
        selectionBackground: 'rgba(124, 77, 255, 0.3)',
      },
      fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
      fontSize: 14,
      cursorBlink: true,
    });

    fitAddon.current = new FitAddon();
    term.current.loadAddon(fitAddon.current);
    term.current.open(terminalRef.current);
    fitAddon.current.fit();

    let active = true;
    const unlistenPromises: Promise<UnlistenFn>[] = [];

    const initPty = async () => {
      // 1. Listen for PTY output via Native Event Bus
      const p1 = listen<string>('pty-output', (event) => {
        if (!active) return;
        // Rust sends Base64 encoded binary stream to preserve ANSI
        const binaryString = atob(event.payload);
        const bytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
          bytes[i] = binaryString.charCodeAt(i);
        }
        term.current?.write(bytes);
      });
      unlistenPromises.push(p1);

      const p2 = listen('pty-exit', () => {
        if (!active) return;
        term.current?.write('\r\n\x1b[33m[Process Exited]\x1b[0m\r\n');
      });
      unlistenPromises.push(p2);

      await Promise.all([p1, p2]);

      if (!active) return;

      // 2. Handle keystrokes (send to PTY stdin)
      term.current?.onData((data) => {
        if (!active) return;
        invoke('write_pty', { data }).catch(console.error);
      });

      // 3. Handle Escape key to blur terminal and focus search bar
      term.current?.attachCustomKeyEventHandler((e) => {
        if (!active) return true;
        if (e.key === 'Escape') {
          document.getElementById('main-search-input')?.focus();
          return false;
        }
        return true;
      });

      // 4. Spawn the PTY Process
      try {
        const { cols, rows } = term.current!;
        await invoke('spawn_pty', { command, cols, rows });
      } catch (e) {
        if (!active) return;
        console.error('Failed to spawn PTY', e);
        term.current?.write(`\r\n\x1b[31mError spawning PTY: ${e}\x1b[0m\r\n`);
      }
      
      if (!active) return;
      // Auto-focus terminal so user can start typing immediately
      term.current?.focus();
    };

    initPty();

    const handleResize = () => {
      if (fitAddon.current && term.current) {
        fitAddon.current.fit();
        const { cols, rows } = term.current;
        invoke('resize_pty', { cols, rows }).catch(console.error);
      }
    };
    window.addEventListener('resize', handleResize);

    return () => {
      active = false;
      window.removeEventListener('resize', handleResize);
      Promise.all(unlistenPromises).then((fns) => {
        fns.forEach((fn) => fn());
      });
      term.current?.dispose();
      term.current = null;
      isMounted.current = false;
      invoke('kill_pty').catch(console.error);
    };
  }, [command]);

  return (
    <div className="terminal-panel-wrapper">
      <div className="terminal-header">
        <span className="terminal-dot red"></span>
        <span className="terminal-dot yellow"></span>
        <span className="terminal-dot green"></span>
        <span className="terminal-title">Terminal: {command}</span>
      </div>
      <div className="terminal-panel" ref={terminalRef} />
    </div>
  );
};

export default TerminalPanel;
