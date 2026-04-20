import { Command } from 'commander';
import { writeFileSync } from 'fs';
import { loadConfig } from '../config';
import { getNodeList } from '../api';

export const nodesCommand = new Command('nodes')
  .description('Fetch and display node configuration for a WebApp')
  .option('-w, --webapp-id <id>', 'WebApp ID')
  .option('-k, --api-key <key>', 'API Key')
  .option('-o, --output <file>', 'Save to JSON file')
  .action(async (options) => {
    const config = loadConfig();
    const apiKey = options.apiKey || config.apiKey;
    const webappId = options.webappId || config.webappId;
    if (!apiKey || !webappId) {
      console.error('Error: API key and WebApp ID required.');
      process.exit(1);
    }
    const { nodes, appInfo } = await getNodeList(apiKey, webappId);
    console.log(`App: ${appInfo?.webappName || 'Unknown'}`);
    console.log(`Nodes (${nodes.length}):`);
    for (const node of nodes) {
      console.log(`  [${node.nodeId}] ${node.nodeName}.${node.fieldName} (${node.fieldType}) = ${node.fieldValue}`);
    }
    if (options.output) {
      writeFileSync(options.output, JSON.stringify(nodes, null, 2));
      console.log(`Saved to ${options.output}`);
    }
  });
