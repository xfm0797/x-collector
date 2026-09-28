import { invoke } from '@tauri-apps/api/core';
import type { RewriteOptions } from '@/types/rewrite';

/** 设置服务 */

export async function getSettings(): Promise<Record<string, string>> {
  return invoke('get_settings');
}

export async function saveSettings(settings: Record<string, string>): Promise<void> {
  return invoke('save_settings', { settings });
}

export interface CollectSettingsVo {
  timeout: number;
  concurrency: number;
  interval_min: number;
  interval_max: number;
  retries: number;
  user_agent: string;
  proxy_url: string;
}

export async function getCollectSettings(): Promise<CollectSettingsVo> {
  return invoke('get_collect_settings');
}

export type { RewriteOptions };
