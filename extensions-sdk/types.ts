import React from 'react';

export interface CrestExtension {
  id: string;
  name: string;
  description: string;
  icon: string; // emoji or icon name
  commands: CrestCommand[];
}

export interface CrestCommand {
  id: string;
  title: string;
  subtitle?: string;
  keywords?: string[];
  onRun: (args: CommandArgs) => Promise<CommandResult>;
  view?: () => React.ReactElement; // Full extension UI
}

export interface CommandArgs {
  query: string;
  [key: string]: any;
}

export interface CommandResult {
  type: 'text' | 'list' | 'detail' | 'toast';
  content: any;
}
