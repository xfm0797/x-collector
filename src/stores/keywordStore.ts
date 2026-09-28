import { create } from 'zustand';
import type { KeywordTask, KeywordTaskInput } from '@/types/keyword';
import type { CollectSummary } from '@/types/common';
import * as keywordService from '@/services/keywordService';

interface KeywordStore {
  tasks: KeywordTask[];
  loading: boolean;
  runningTaskId: number | null;
  lastSummary: CollectSummary | null;
  fetchTasks: () => Promise<void>;
  createTask: (input: KeywordTaskInput) => Promise<void>;
  updateTask: (id: number, input: Partial<KeywordTaskInput>) => Promise<void>;
  deleteTasks: (ids: number[]) => Promise<void>;
  runTask: (id: number) => Promise<CollectSummary>;
}

export const useKeywordStore = create<KeywordStore>((set, get) => ({
  tasks: [],
  loading: false,
  runningTaskId: null,
  lastSummary: null,

  fetchTasks: async () => {
    set({ loading: true });
    try {
      const tasks = await keywordService.listKeywordTasks();
      set({ tasks });
    } finally {
      set({ loading: false });
    }
  },

  createTask: async (input) => {
    await keywordService.createKeywordTask(input);
    await get().fetchTasks();
  },

  updateTask: async (id, input) => {
    await keywordService.updateKeywordTask(id, input);
    await get().fetchTasks();
  },

  deleteTasks: async (ids) => {
    await keywordService.deleteKeywordTasks(ids);
    await get().fetchTasks();
  },

  runTask: async (id) => {
    set({ runningTaskId: id, lastSummary: null });
    try {
      const summary = await keywordService.runKeywordTask(id);
      set({ lastSummary: summary });
      await get().fetchTasks();
      return summary;
    } finally {
      set({ runningTaskId: null });
    }
  },
}));
