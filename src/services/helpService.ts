import { invoke } from '@tauri-apps/api/core';
import type {
  ManualSection,
  ManualContent,
  ConfigExample,
  FaqCategory,
  FaqItem,
  SearchResult,
  QuickStartGuide,
  ShortcutItem,
  ErrorCode,
  UpdateInfo,
} from '@/types/help';

/** 帮助系统服务 */

export async function getManualIndex(): Promise<ManualSection[]> {
  return invoke('get_manual_index');
}

export async function getManualSection(path: string): Promise<ManualContent> {
  return invoke('get_manual_section', { path });
}

export async function getExamples(platform?: string): Promise<ConfigExample[]> {
  return invoke('get_examples', { platform });
}

export async function getFaqCategories(): Promise<FaqCategory[]> {
  return invoke('get_faq_categories');
}

export async function getFaqs(category?: string): Promise<FaqItem[]> {
  return invoke('get_faqs', { category });
}

export async function searchHelp(query: string): Promise<SearchResult[]> {
  return invoke('search_help', { query });
}

export async function getQuickStart(): Promise<QuickStartGuide> {
  return invoke('get_quick_start');
}

export async function getChangelog(): Promise<string> {
  return invoke('get_changelog');
}

export async function getShortcuts(): Promise<ShortcutItem[]> {
  return invoke('get_shortcuts');
}

export async function getErrorCodes(): Promise<ErrorCode[]> {
  return invoke('get_error_codes');
}

export async function checkUpdate(): Promise<UpdateInfo> {
  return invoke('check_update');
}

export async function openExternalLink(url: string): Promise<void> {
  return invoke('open_external_link', { url });
}
