import { Command } from 'commander';
import { loadConfig } from '../config';
import { queryTaskOutputs, buildFileUrl } from '../api';

export const statusCommand = new Command('status')
  .description('Query task status and outputs')
  .requiredOption('-t, --task-id <id>', 'Task ID')
  .option('-k, --api-key <key>', 'API Key')
  .action(async (options) => {
    const config = loadConfig();
    const apiKey = options.apiKey || config.apiKey;
    if (!apiKey) {
      console.error('Error: API key required.');
      process.exit(1);
    }
    const result = await queryTaskOutputs(apiKey, options.taskId);
    console.log(JSON.stringify(result, null, 2));
    if (Array.isArray(result.data) && result.data.length > 0) {
      console.log('\nOutputs:');
      for (const output of result.data) {
        console.log(`  - ${buildFileUrl(output.fileUrl)}`);
      }
    }
  });
