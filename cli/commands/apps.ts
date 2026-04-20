import { Command } from 'commander';
import chalk from 'chalk';
import { getOfficialAppList } from '../api';

export const appsCommand = new Command('apps')
  .description('List official RunningHub apps')
  .option('-s, --search <keyword>', 'Search keyword')
  .option('-p, --page <n>', 'Page number', '1')
  .option('--size <n>', 'Page size', '20')
  .action(async (options) => {
    const { records, total } = await getOfficialAppList(
      parseInt(options.page),
      parseInt(options.size),
      'RECOMMEND',
      options.search
    );
    console.log(chalk.bold(`Total: ${total}, Page: ${options.page}`));
    for (const app of records) {
      console.log(`\n${chalk.cyan(`[${app.id}]`)} ${chalk.bold(app.name)}`);
      console.log(`  ${app.intro}`);
      if (app.authorInfo) console.log(`  Author: ${app.authorInfo.name}`);
      if (app.statisticsInfo) {
        const s = app.statisticsInfo;
        console.log(`  Likes: ${s.likeCount} | Uses: ${s.useCount}`);
      }
    }
  });
