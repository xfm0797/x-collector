import { useEffect } from 'react';
import { usePublishStore } from '@/stores/publishStore';

/** 发布管理 Hook */
export function usePublish() {
  const {
    connections,
    records,
    loading,
    testingId,
    publishing,
    fetchConnections,
    fetchRecords,
    saveConnection,
    deleteConnections,
    testConnection,
    publishArticle,
    batchPublish,
  } = usePublishStore();

  useEffect(() => {
    void fetchConnections();
    void fetchRecords();
  }, [fetchConnections, fetchRecords]);

  return {
    connections,
    records,
    loading,
    testingId,
    publishing,
    fetchConnections,
    fetchRecords,
    saveConnection,
    deleteConnections,
    testConnection,
    publishArticle,
    batchPublish,
  };
}
