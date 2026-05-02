import { CommandArgs, CommandResult, CrestCommand, CrestExtension } from './types';

// The global API exposed to extensions via context isolation
export const crest = {
  // Post message to the Tauri backend
  invoke: async (command: string, args: any): Promise<any> => {
    // @ts-ignore - Tauri window API
    if (window.__TAURI__ && window.__TAURI__.core) {
      // @ts-ignore
      return await window.__TAURI__.core.invoke(command, args);
    }
    console.warn(`Mocking invoke for ${command}`);
    return null;
  },

  // Extension runtime context
  registerExtension: (extension: CrestExtension) => {
    console.log(`Registered extension: ${extension.name}`);
    // Register commands in the global namespace or send to backend
  }
};
