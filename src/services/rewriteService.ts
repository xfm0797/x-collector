import { invoke } from '@tauri-apps/api/core';
import type { RewriteOptions, RewritePreview } from '@/types/rewrite';
import type { Article } from '@/types/article';
import type { OpResult } from '@/types/common';

/** 伪原创服务 */

export async function getRewriteOptions(): Promise<RewriteOptions> {
  return invoke('get_rewrite_options');
}

export async function saveRewriteOptions(options: RewriteOptions): Promise<void> {
  return invoke('save_rewrite_options', { options });
}

/** 预览改写效果（不落库） */
export async function previewRewrite(articleId: number): Promise<RewritePreview> {
  return invoke('preview_rewrite', { articleId });
}

/** 应用伪原创到文章 */
export async function applyRewrite(articleId: number): Promise<Article> {
  return invoke('apply_rewrite', { articleId });
}

/** 导入自定义同义词词库（JSON：[[词, 同义词...], ...]） */
export async function importCustomDict(content: string): Promise<OpResult> {
  return invoke('import_custom_dict', { content });
}

export async function resetCustomDict(): Promise<void> {
  return invoke('reset_custom_dict');
}

export async function getDictStats(): Promise<{ builtin_groups: number; custom_groups: number }> {
  return invoke('get_dict_stats');
}
