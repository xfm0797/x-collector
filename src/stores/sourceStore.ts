import { create } from 'zustand';
import type { CollectSource, CollectSourceInput } from '@/types/keyword';
import type { CollectSummary } from '@/types/common';
import * as collectService from '@/services/collectService';

interface SourceStore {
  sources: CollectSource[];
  loading: boolean;
  collectingSourceId: number | null;
  lastSummary: CollectSummary | null;
  fetchSources: () => Promise<void>;
  createSource: (input: CollectSourceInput) => Promise<void>;
  updateSource: (id: number, input: Partial<CollectSourceInput>) => Promise<void>;
  deleteSources: (ids: number[]) => Promise<void>;
  collectFromSource: (id: number) => Promise<CollectSummary>;
}

export const useSourceStore = create<SourceStore>((set, get) => ({
  sources: [],
  loading: false,
  collectingSourceId: null,
  lastSummary: null,

  fetchSources: async () => {
    set({ loading: true });
    try {
      const sources = await collectService.listSources();
      set({ sources });
    } finally {
      set({ loading: false });
    }
  },

  createSource: async (input) => {
    await collectService.createSource(input);
    await get().fetchSources();
  },

  updateSource: async (id, input) => {
    await collectService.updateSource(id, input);
    await get().fetchSources();
  },

  deleteSources: async (ids) => {
    await collectService.deleteSources(ids);
    await get().fetchSources();
  },

  collectFromSource: async (id) => {
    set({ collectingSourceId: id, lastSummary: null });
    try {
      const summary = await collectService.collectFromSource(id);
      set({ lastSummary: summary });
      await get().fetchSources();
      return summary;
    } finally {
      set({ collectingSourceId: null });
    }
  },
}));
