import { useEffect } from 'react';
import { useKeywordStore } from '@/stores/keywordStore';

/** 关键词任务 Hook */
export function useKeywords() {
  const { tasks, loading, runningTaskId, lastSummary, fetchTasks, createTask, updateTask, deleteTasks, runTask } =
    useKeywordStore();

  useEffect(() => {
    void fetchTasks();
  }, [fetchTasks]);

  return { tasks, loading, runningTaskId, lastSummary, fetchTasks, createTask, updateTask, deleteTasks, runTask };
}
