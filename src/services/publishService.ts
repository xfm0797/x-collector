import { invoke } from '@tauri-apps/api/core';
import type { CmsConnection, CmsConnectionInput, PublishRecord, PublishTarget, HexoExportResult } from '@/types/publish';
import type { OpResult } from '@/types/common';

/** 发布服务 */

export async function listCmsConnections(): Promise<CmsConnection[]> {
  return invoke('list_cms_connections');
}

export async function saveCmsConnection(input: CmsConnectionInput): Promise<CmsConnection> {
  return invoke('save_cms_connection', { input });
}

export async function deleteCmsConnections(ids: number[]): Promise<void> {
  return invoke('delete_cms_connections', { ids });
}

export async function testCmsConnection(id: number): Promise<OpResult> {
  return invoke('test_cms_connection', { cmsId: id });
}

/** 单篇发布 */
export async function publishArticle(articleId: number, target: PublishTarget): Promise<PublishRecord> {
  return invoke('publish_article', { articleId, target });
}

/** 批量发布 */
export async function batchPublish(articleIds: number[], target: PublishTarget): Promise<PublishRecord[]> {
  return invoke('batch_publish', { articleIds, target });
}

/** 批量导出 Hexo */
export async function exportHexo(articleIds: number[], outputDir?: string): Promise<HexoExportResult> {
  return invoke('export_hexo', { articleIds, outputDir });
}

export async function listPublishRecords(limit = 100): Promise<PublishRecord[]> {
  return invoke('list_publish_records', { limit });
}
