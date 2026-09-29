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

// ── 可视化选择器 ──────────────────────────────────────────

export interface DomNode {
  tag: string;
  id: string | null;
  classes: string[];
  css: string;
  path: string;
  text_preview: string | null;
  text_length: number;
  children: DomNode[];
}

export interface PageInspect {
  final_url: string;
  title: string;
  dom: DomNode;
}

export interface SelectorMatch {
  index: number;
  text_preview: string;
  html_length: number;
}

export interface SelectorTestResult {
  selector: string;
  matched: number;
  matches: SelectorMatch[];
}

/** 抓取页面并返回 DOM 结构树 */
export async function inspectPage(url: string): Promise<PageInspect> {
  return invoke('inspect_page', { url });
}

/** 在页面上测试 CSS 选择器 */
export async function testSelector(url: string, css: string): Promise<SelectorTestResult> {
  return invoke('test_selector', { url, css });
}
