import { invoke } from '@tauri-apps/api/core';
import type { KeywordTask, KeywordTaskInput } from '@/types/keyword';
import type { CollectSummary } from '@/types/common';

/** 关键词任务服务 */

export async function listKeywordTasks(): Promise<KeywordTask[]> {
  return invoke('list_keyword_tasks');
}

export async function createKeywordTask(input: KeywordTaskInput): Promise<KeywordTask> {
  return invoke('create_keyword_task', { input });
}

export async function updateKeywordTask(id: number, input: Partial<KeywordTaskInput>): Promise<void> {
  return invoke('update_keyword_task', { id, input });
}

export async function deleteKeywordTasks(ids: number[]): Promise<void> {
  return invoke('delete_keyword_tasks', { ids });
}

/** 执行一次关键词采集任务 */
export async function runKeywordTask(id: number): Promise<CollectSummary> {
  return invoke('run_keyword_task', { taskId: id });
}

/** 采集日志 */
export interface CollectLog {
  id: number;
  task_id?: number | null;
  task_type: string;
  keyword?: string;
  url?: string;
  status: string;
  message?: string;
  created_at: string;
}

export async function listCollectLogs(limit = 100): Promise<CollectLog[]> {
  return invoke('list_collect_logs', { limit });
}
