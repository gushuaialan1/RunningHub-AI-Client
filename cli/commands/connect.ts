import { Command } from 'commander';
import { saveConfig } from '../config';

export const connectCommand = new Command('connect')
  .description('Save API key and WebApp ID to local config')
  .requiredOption('-k, --api-key <key>', 'RunningHub API Key')
  .requiredOption('-w, --webapp-id <id>', 'WebApp ID')
  .action((options) => {
    saveConfig({ apiKey: options.apiKey, webappId: options.webappId });
    console.log('Configuration saved to ~/.runninghub/config.json');
  });
