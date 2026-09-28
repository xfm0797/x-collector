import { invoke } from '@tauri-apps/api/core';
import type { CollectSource, CollectSourceInput } from '@/types/keyword';
import type { CollectSummary } from '@/types/common';

/** 采集源服务 */

export async function listSources(): Promise<CollectSource[]> {
  return invoke('list_sources');
}

export async function createSource(input: CollectSourceInput): Promise<CollectSource> {
  return invoke('create_source', { input });
}

export async function updateSource(id: number, input: Partial<CollectSourceInput>): Promise<void> {
  return invoke('update_source', { id, input });
}

export async function deleteSources(ids: number[]): Promise<void> {
  return invoke('delete_sources', { ids });
}

/** 从采集源执行一次采集 */
export async function collectFromSource(id: number): Promise<CollectSummary> {
  return invoke('collect_from_source', { sourceId: id });
}

/** 从 sitemap.xml 批量发现链接 */
export async function discoverFromSitemap(url: string): Promise<string[]> {
  return invoke('discover_from_sitemap', { url });
}
