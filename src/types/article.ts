/** 文章相关类型 */

export type ArticleStatus = 'collected' | 'edited' | 'published' | 'failed';

export interface Article {
  id: number;
  title: string;
  url: string;
  source?: string;
  author?: string;
  content_html?: string;
  content_md?: string;
  excerpt?: string;
  tags?: string[] | null;
  category?: string;
  cover_image?: string;
  status: ArticleStatus;
  is_draft: number;
  collected_at?: string;
  updated_at?: string;
  published_at?: string;
}

export interface ArticleInput {
  title: string;
  url: string;
  source?: string;
  author?: string;
  content_md?: string;
  excerpt?: string;
  tags?: string[];
  category?: string;
  cover_image?: string;
}

export interface ArticleQuery {
  keyword?: string;
  status?: string;
  source?: string;
  page?: number;
  page_size?: number;
  sort?: 'collected_at' | 'title' | 'status';
  order?: 'asc' | 'desc';
}
