import { useEffect, useCallback } from 'react';
import { useArticleStore } from '@/stores/articleStore';

/** 文章列表 Hook */
export function useArticles() {
  const {
    articles,
    total,
    page,
    pageSize,
    query,
    loading,
    setQuery,
    fetchArticles,
    deleteArticles,
    updateStatus,
  } = useArticleStore();

  useEffect(() => {
    void fetchArticles();
  }, [fetchArticles, page, query]);

  const refresh = useCallback(() => void fetchArticles(), [fetchArticles]);

  return {
    articles,
    total,
    page,
    pageSize,
    query,
    loading,
    setQuery,
    refresh,
    deleteArticles,
    updateStatus,
  };
}
