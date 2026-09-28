import { invoke } from '@tauri-apps/api/core';
import type { Article, ArticleInput, ArticleQuery } from '@/types/article';
import type { PageData, CollectSummary } from '@/types/common';

/** 文章服务：封装文章相关 Tauri 命令 */

export async function listArticles(query: ArticleQuery): Promise<PageData<Article>> {
  return invoke('list_articles', { query });
}

export async function getArticle(id: number): Promise<Article> {
  return invoke('get_article', { id });
}

export async function createArticle(input: ArticleInput): Promise<Article> {
  return invoke('create_article', { input });
}

export async function updateArticle(id: number, input: Partial<ArticleInput>): Promise<void> {
  return invoke('update_article', { id, input });
}

export async function updateArticleStatus(id: number, status: string): Promise<void> {
  return invoke('update_article_status', { id, status });
}

export async function deleteArticles(ids: number[]): Promise<void> {
  return invoke('delete_articles', { ids });
}

/** 单页 URL 采集 */
export async function collectSinglePage(url: string): Promise<Article> {
  return invoke('collect_single_page', { url });
}

export interface DashboardStats {
  article_total: number;
  today_collected: number;
  pending_publish: number;
  source_total: number;
  keyword_total: number;
  published_total: number;
  publish_by_type: { publish_type: string; count: number }[];
  recent_articles: Pick<Article, 'id' | 'title' | 'source' | 'collected_at' | 'status'>[];
}

export async function getDashboardStats(): Promise<DashboardStats> {
  return invoke('get_dashboard_stats');
}

export type { CollectSummary };
