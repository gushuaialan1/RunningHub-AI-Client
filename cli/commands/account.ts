import { Command } from 'commander';
import chalk from 'chalk';
import { loadConfig } from '../config';
import { getAccountInfo } from '../api';

export const accountCommand = new Command('account')
  .description('Show account information')
  .option('-k, --api-key <key>', 'API Key')
  .action(async (options) => {
    const config = loadConfig();
    const apiKey = options.apiKey || config.apiKey;
    if (!apiKey) {
      console.error(chalk.red('Error: API key required.'));
      process.exit(1);
    }
    const info = await getAccountInfo(apiKey);
    console.log(`${chalk.bold('Remaining Coins:')} ${info.remainCoins}`);
    console.log(`${chalk.bold('Current Tasks:')} ${info.currentTaskCounts}`);
    if (info.remainMoney) console.log(`${chalk.bold('Remaining Money:')} ${info.remainMoney} ${info.currency || ''}`);
    console.log(`${chalk.bold('API Type:')} ${info.apiType}`);
  });
