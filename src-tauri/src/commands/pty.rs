use tauri::{AppHandle, Emitter};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use std::io::{Read, Write};

pub struct PtyHandle {
    pub master: Box<dyn MasterPty + Send>,
    pub writer: Box<dyn Write + Send>,
}

static PTY_MASTER: Lazy<Arc<Mutex<Option<PtyHandle>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

#[tauri::command]
pub fn spawn_pty(app: AppHandle, command: String, cols: u16, rows: u16) -> Result<(), String> {
    let _ = kill_pty();

    let pty_system = NativePtySystem::default();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }).map_err(|e| e.to_string())?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    let cmd_builder = CommandBuilder::new(&shell);
    
    // Always spawn an interactive shell. We do not use -c so the shell stays alive.

    // Spawn the child process inside the pseudo-terminal
    let _child = pair.slave.spawn_command(cmd_builder).map_err(|e| e.to_string())?;

    let master = pair.master;
    let mut reader = master.try_clone_reader().map_err(|e| e.to_string())?;
    let mut writer = master.take_writer().map_err(|e| e.to_string())?;

    // If the user typed a command in the search bar, inject it directly into the shell's stdin
    if !command.is_empty() {
        let cmd_with_newline = format!("{}\n", command);
        let _ = writer.write_all(cmd_with_newline.as_bytes());
        let _ = writer.flush();
    }

    if let Ok(mut guard) = PTY_MASTER.lock() {
        *guard = Some(PtyHandle { master, writer });
    }

    // Native Event Bus Thread: Stream PTY stdout to React continuously
    std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let data = buf[..n].to_vec();
                    // We Base64 encode the terminal binary stream to preserve ANSI codes safely
                    use base64::{Engine as _, engine::general_purpose::STANDARD};
                    let b64 = STANDARD.encode(&data);
                    let _ = app.emit("pty-output", b64);
                }
                Ok(_) => break, // EOF (process died gracefully)
                Err(_) => break, // Error / disconnected
            }
        }
        let _ = app.emit("pty-exit", ());
    });

    Ok(())
}

#[tauri::command]
pub fn write_pty(data: String) -> Result<(), String> {
    if let Ok(mut guard) = PTY_MASTER.lock() {
        if let Some(handle) = guard.as_mut() {
            let _ = handle.writer.write_all(data.as_bytes());
            let _ = handle.writer.flush();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn resize_pty(cols: u16, rows: u16) -> Result<(), String> {
    if let Ok(guard) = PTY_MASTER.lock() {
        if let Some(handle) = guard.as_ref() {
            let _ = handle.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
    }
    Ok(())
}

#[tauri::command]
pub fn kill_pty() -> Result<(), String> {
    if let Ok(mut guard) = PTY_MASTER.lock() {
        *guard = None; // Dropping the master closes the PTY and kills the child process
    }
    Ok(())
}
