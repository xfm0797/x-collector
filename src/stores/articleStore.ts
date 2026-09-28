import { create } from 'zustand';
import type { Article, ArticleQuery } from '@/types/article';
import type { PageData } from '@/types/common';
import * as articleService from '@/services/articleService';

interface ArticleStore {
  articles: Article[];
  total: number;
  page: number;
  pageSize: number;
  query: ArticleQuery;
  loading: boolean;
  current: Article | null;
  setQuery: (query: Partial<ArticleQuery>) => void;
  fetchArticles: () => Promise<void>;
  fetchArticle: (id: number) => Promise<Article>;
  deleteArticles: (ids: number[]) => Promise<void>;
  updateStatus: (id: number, status: string) => Promise<void>;
  updateArticle: (id: number, input: Partial<Article>) => Promise<void>;
  collectSinglePage: (url: string) => Promise<Article>;
}

export const useArticleStore = create<ArticleStore>((set, get) => ({
  articles: [],
  total: 0,
  page: 1,
  pageSize: 10,
  query: {},
  loading: false,
  current: null,

  setQuery: (query) => {
    const merged = { ...get().query, ...query };
    set({ query: merged, page: query.page ?? 1 });
  },

  fetchArticles: async () => {
    const { query, page, pageSize } = get();
    set({ loading: true });
    try {
      const data: PageData<Article> = await articleService.listArticles({
        ...query,
        page,
        page_size: pageSize,
      });
      set({ articles: data.list, total: data.total });
    } finally {
      set({ loading: false });
    }
  },

  fetchArticle: async (id) => {
    const article = await articleService.getArticle(id);
    set({ current: article });
    return article;
  },

  deleteArticles: async (ids) => {
    await articleService.deleteArticles(ids);
    await get().fetchArticles();
  },

  updateStatus: async (id, status) => {
    await articleService.updateArticleStatus(id, status);
    await get().fetchArticles();
  },

  updateArticle: async (id, input) => {
    await articleService.updateArticle(id, {
      title: input.title,
      url: input.url,
      content_md: input.content_md,
      tags: input.tags ?? undefined,
      category: input.category,
      excerpt: input.excerpt,
      author: input.author,
    });
  },

  collectSinglePage: async (url) => articleService.collectSinglePage(url),
}));
