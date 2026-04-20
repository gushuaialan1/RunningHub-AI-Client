import { readFileSync } from 'fs';
import { basename } from 'path';
import type { ApiResponse, NodeInfo, SubmitTaskData, UploadData, TaskOutput, WebAppInfo } from '../types';

const API_HOST = 'https://www.runninghub.cn';

async function handleResponse<T>(response: Response): Promise<ApiResponse<T>> {
  if (!response.ok) {
    throw new Error(`HTTP Error: ${response.status} ${response.statusText}`);
  }
  return await response.json();
}

export const buildFileUrl = (value: string): string => {
  if (!value) return '';
  if (/^(https?:\/\/|data:)/i.test(value)) return value;
  return `https://www.runninghub.cn/task/openapi/view/${value}`;
};

export interface GetNodeListResult {
  nodes: NodeInfo[];
  appInfo: WebAppInfo | null;
}

export const getNodeList = async (apiKey: string, webappId: string): Promise<GetNodeListResult> => {
  const url = `${API_HOST}/api/webapp/apiCallDemo?apiKey=${apiKey}&webappId=${webappId}`;
  const response = await fetch(url, {
    method: 'GET',
    headers: { 'Accept': 'application/json' },
  });
  const json: ApiResponse<any> = await handleResponse(response);
  if (json.code !== 0 || !json.data?.nodeInfoList) {
    throw new Error(json.msg || 'Failed to fetch node list');
  }
  const nodes = json.data.nodeInfoList.map((node: any) => ({
    ...node,
    fieldData: node.fieldData || node.field_data || node.options || undefined
  }));
  const appInfo: WebAppInfo | null = json.data.webappName ? {
    webappName: json.data.webappName,
    description: json.data.description || '',
    descriptionEn: json.data.descriptionEn,
    covers: json.data.covers,
    tags: json.data.tags,
    statisticsInfo: json.data.statisticsInfo,
  } : null;
  return { nodes, appInfo };
};

export const uploadFile = async (apiKey: string, filePath: string): Promise<UploadData> => {
  const url = `${API_HOST}/task/openapi/upload`;
  const buffer = readFileSync(filePath);
  const blob = new Blob([buffer]);
  const formData = new FormData();
  formData.append('apiKey', apiKey);
  formData.append('fileType', 'input');
  formData.append('file', blob, basename(filePath));

  const response = await fetch(url, { method: 'POST', body: formData });
  const json: ApiResponse<UploadData> = await handleResponse(response);
  if (json.code !== 0) {
    throw new Error(json.msg || 'Upload failed');
  }
  return json.data;
};

export const submitTask = async (apiKey: string, webappId: string, nodeInfoList: NodeInfo[]): Promise<SubmitTaskData> => {
  const url = `${API_HOST}/task/openapi/ai-app/run`;
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ webappId, apiKey, nodeInfoList }),
  });
  const json: ApiResponse<SubmitTaskData> = await handleResponse(response);
  if (json.code !== 0) {
    throw new Error(json.msg || 'Submission failed');
  }
  return json.data;
};

export const queryTaskOutputs = async (apiKey: string, taskId: string): Promise<ApiResponse<TaskOutput[] | any>> => {
  const url = `${API_HOST}/task/openapi/outputs`;
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ apiKey, taskId }),
  });
  return await handleResponse(response);
};

export const getAccountInfo = async (apiKey: string) => {
  const url = `${API_HOST}/uc/openapi/accountStatus`;
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Host': 'www.runninghub.cn',
    },
    body: JSON.stringify({ apikey: apiKey }),
  });
  const json = await handleResponse(response);
  if (json.code !== 0) {
    throw new Error(json.msg || 'Failed to get account info');
  }
  return json.data;
};

export interface AppListItem {
  id: string;
  name: string;
  intro: string;
  covers: { fileUri: string; thumbnailUri: string; imageWidth: number; imageHeight: number; type: string; }[];
  authorInfo?: { name: string; avatar: string; };
  statisticsInfo?: { viewCount: number | string; useCount: number | string; likeCount: number | string; collectCount: number | string; };
}

export const getOfficialAppList = async (page: number = 1, size: number = 50, sort: string = 'RECOMMEND', search?: string) => {
  const url = `${API_HOST}/api/webapp/list`;
  const body: any = { current: page, size, carefullyChosen: true, sort };
  if (search?.trim()) body.search = search.trim();
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  const json = await handleResponse(response);
  if (json.code !== 0) throw new Error(json.msg || 'Failed to fetch app list');
  return { records: json.data.records as AppListItem[], total: parseInt(json.data.total) || 0 };
};

export const getAppDetailById = async (appId: string): Promise<AppListItem | null> => {
  const url = `${API_HOST}/api/webapp/list`;
  const body = { current: 1, size: 1, search: appId };
  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    const json = await handleResponse(response);
    if (json.code !== 0 || !json.data?.records?.length) return null;
    const app = json.data.records[0] as AppListItem;
    return app.id === appId ? app : null;
  } catch {
    return null;
  }
};
