import { Command } from 'commander';
import { readFileSync } from 'fs';
import chalk from 'chalk';
import { loadConfig } from '../config';
import { getNodeList, submitTask, buildFileUrl } from '../api';
import { resolveNodeFiles, pollTask, downloadFile, getOutputPath } from '../utils';
import type { NodeInfo } from '../../types';

export const runCommand = new Command('run')
  .description('Run a single task')
  .option('-w, --webapp-id <id>', 'WebApp ID')
  .option('-k, --api-key <key>', 'API Key')
  .option('-n, --nodes <file>', 'JSON file with NodeInfo list')
  .option('-o, --output <dir>', 'Output directory', './output')
  .action(async (options) => {
    const config = loadConfig();
    const apiKey = options.apiKey || config.apiKey;
    const webappId = options.webappId || config.webappId;
    if (!apiKey || !webappId) {
      console.error(chalk.red('Error: API key and WebApp ID required. Use "connect" command or pass as options.'));
      process.exit(1);
    }

    let nodes: NodeInfo[];
    if (options.nodes) {
      nodes = JSON.parse(readFileSync(options.nodes, 'utf-8'));
    } else {
      const result = await getNodeList(apiKey, webappId);
      nodes = result.nodes;
      console.log(chalk.blue(`Using app: ${result.appInfo?.webappName || 'Unknown'}`));
      console.log(chalk.gray('Tip: Use --nodes to provide custom parameters, or "rh-cli nodes" to see available nodes'));
    }

    nodes = await resolveNodeFiles(apiKey, nodes);

    console.log(chalk.blue('Submitting task...'));
    const { taskId, promptTips } = await submitTask(apiKey, webappId, nodes);
    console.log(chalk.green(`Task ID: ${taskId}`));
    if (promptTips) {
      console.log(chalk.yellow('Prompt tips:'), promptTips);
    }

    const outputs = await pollTask(apiKey, taskId);
    console.log(chalk.green(`Task completed! ${outputs.length} output(s)`));

    for (let i = 0; i < outputs.length; i++) {
      const url = buildFileUrl(outputs[i].fileUrl);
      const path = getOutputPath(options.output, 0, i, outputs[i].fileType);
      console.log(chalk.blue(`Downloading: ${url}`));
      await downloadFile(url, path);
      console.log(chalk.green(`Saved: ${path}`));
    }
  });
