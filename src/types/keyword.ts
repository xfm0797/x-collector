/** 关键词任务与采集源类型 */

export type SearchEngine = 'baidu' | 'google' | 'bing' | 'sogou' | 'custom';
export type MatchMode = 'title' | 'content' | 'title_or_content' | 'exact';

export interface KeywordTask {
  id: number;
  keyword: string;
  group_name: string;
  search_engine: SearchEngine;
  site_limit?: string;
  match_mode: MatchMode;
  max_pages: number;
  interval_minutes: number;
  enabled: number;
  last_run?: string;
  created_at?: string;
}

export interface KeywordTaskInput {
  keyword: string;
  group_name?: string;
  search_engine?: SearchEngine;
  site_limit?: string;
  match_mode?: MatchMode;
  max_pages?: number;
  interval_minutes?: number;
  enabled?: number;
}

export type SourceType = 'rss' | 'webpage' | 'sitemap' | 'api';

export interface CollectSource {
  id: number;
  name: string;
  url: string;
  source_type: SourceType;
  selector_title?: string;
  selector_content?: string;
  selector_author?: string;
  selector_date?: string;
  selector_tags?: string;
  selector_cover?: string;
  remove_selectors?: string[] | null;
  group_name: string;
  enabled: number;
  last_collected?: string;
  created_at?: string;
}

export interface CollectSourceInput {
  name: string;
  url: string;
  source_type: SourceType;
  selector_title?: string;
  selector_content?: string;
  selector_author?: string;
  selector_date?: string;
  selector_tags?: string;
  selector_cover?: string;
  remove_selectors?: string[];
  group_name?: string;
  enabled?: number;
}
