import { existsSync, mkdirSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import type { NodeInfo, TaskOutput } from '../types';
import { uploadFile, buildFileUrl, queryTaskOutputs } from './api';

const uploadedCache = new Map<string, string>();

export async function resolveNodeFiles(apiKey: string, nodes: NodeInfo[]): Promise<NodeInfo[]> {
  const result: NodeInfo[] = [];
  for (const node of nodes) {
    if (['IMAGE', 'AUDIO', 'VIDEO'].includes(node.fieldType)) {
      const value = node.fieldValue;
      if (value && !value.startsWith('http') && existsSync(value)) {
        if (!uploadedCache.has(value)) {
          const data = await uploadFile(apiKey, value);
          uploadedCache.set(value, data.fileName);
        }
        result.push({ ...node, fieldValue: uploadedCache.get(value)! });
        continue;
      }
    }
    result.push(node);
  }
  return result;
}

export async function pollTask(apiKey: string, taskId: string, interval = 3000): Promise<TaskOutput[]> {
  while (true) {
    await new Promise(r => setTimeout(r, interval));
    const result = await queryTaskOutputs(apiKey, taskId);
    if (result.code !== 0) {
      throw new Error(`Query failed: ${result.msg}`);
    }
    if (Array.isArray(result.data) && result.data.length > 0) {
      return result.data as TaskOutput[];
    }
    if (result.data && typeof result.data === 'object') {
      const status = result.data.status;
      if (status === 'FAILED' || result.data.failedReason) {
        const reason = result.data.failedReason?.exception_message || 'Unknown error';
        throw new Error(`Task failed: ${reason}`);
      }
    }
  }
}

export async function downloadFile(url: string, outputPath: string): Promise<void> {
  mkdirSync(dirname(outputPath), { recursive: true });
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Download failed: ${response.statusText}`);
  const buffer = await response.arrayBuffer();
  writeFileSync(outputPath, Buffer.from(buffer));
}

export function getOutputPath(baseDir: string, taskIndex: number, outputIndex: number, fileType?: string): string {
  const ext = fileType || 'png';
  return join(baseDir, `task_${taskIndex + 1}_output_${outputIndex + 1}.${ext}`);
}
