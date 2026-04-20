#!/usr/bin/env node
import { Command } from 'commander';
import { connectCommand } from './commands/connect';
import { nodesCommand } from './commands/nodes';
import { runCommand } from './commands/run';
import { batchCommand } from './commands/batch';
import { statusCommand } from './commands/status';
import { appsCommand } from './commands/apps';
import { accountCommand } from './commands/account';

const program = new Command();

program
  .name('rh-cli')
  .description('RunningHub AI CLI')
  .version('1.0.0');

program.addCommand(connectCommand);
program.addCommand(nodesCommand);
program.addCommand(runCommand);
program.addCommand(batchCommand);
program.addCommand(statusCommand);
program.addCommand(appsCommand);
program.addCommand(accountCommand);

program.parse();
