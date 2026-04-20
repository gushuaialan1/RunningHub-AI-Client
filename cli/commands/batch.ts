import { Command } from 'commander';
import { readFileSync } from 'fs';
import pLimit from 'p-limit';
import chalk from 'chalk';
import { loadConfig } from '../config';
import { submitTask, buildFileUrl } from '../api';
import { resolveNodeFiles, pollTask, downloadFile, getOutputPath } from '../utils';
import type { NodeInfo } from '../../types';

export const batchCommand = new Command('batch')
  .description('Run batch tasks')
  .requiredOption('-b, --batch <file>', 'JSON file with NodeInfo[][] array')
  .option('-w, --webapp-id <id>', 'WebApp ID')
  .option('-k, --api-key <key>', 'API Key')
  .option('-c, --concurrency <n>', 'Max concurrent tasks', '3')
  .option('-o, --output <dir>', 'Output directory', './output')
  .action(async (options) => {
    const config = loadConfig();
    const apiKey = options.apiKey || config.apiKey;
    const webappId = options.webappId || config.webappId;
    const concurrency = parseInt(options.concurrency);

    if (!apiKey || !webappId) {
      console.error(chalk.red('Error: API key and WebApp ID required.'));
      process.exit(1);
    }

    const batchList: NodeInfo[][] = JSON.parse(readFileSync(options.batch, 'utf-8'));
    console.log(chalk.blue(`Loaded ${batchList.length} tasks, concurrency: ${concurrency}`));

    const limit = pLimit(concurrency);
    let completed = 0;
    let failed = 0;

    const tasks = batchList.map((nodes, index) => limit(async () => {
      console.log(chalk.blue(`[Task ${index + 1}/${batchList.length}] Starting...`));
      try {
        const resolved = await resolveNodeFiles(apiKey, nodes);
        const { taskId } = await submitTask(apiKey, webappId, resolved);
        console.log(chalk.blue(`[Task ${index + 1}] Submitted: ${taskId}`));
        const outputs = await pollTask(apiKey, taskId);
        console.log(chalk.green(`[Task ${index + 1}] Completed: ${outputs.length} output(s)`));
        completed++;
        for (let i = 0; i < outputs.length; i++) {
          const url = buildFileUrl(outputs[i].fileUrl);
          const path = getOutputPath(options.output, index, i, outputs[i].fileType);
          await downloadFile(url, path);
          console.log(chalk.green(`[Task ${index + 1}] Saved: ${path}`));
        }
      } catch (e: any) {
        console.error(chalk.red(`[Task ${index + 1}] Failed: ${e.message}`));
        failed++;
      }
    }));

    await Promise.all(tasks);
    console.log(chalk.bold(`\nBatch complete: ${completed} succeeded, ${failed} failed`));
  });
