import { create } from 'zustand';
import type { CmsConnection, CmsConnectionInput, PublishRecord, PublishTarget } from '@/types/publish';
import * as publishService from '@/services/publishService';

interface PublishStore {
  connections: CmsConnection[];
  records: PublishRecord[];
  loading: boolean;
  testingId: number | null;
  publishing: boolean;
  fetchConnections: () => Promise<void>;
  fetchRecords: () => Promise<void>;
  saveConnection: (input: CmsConnectionInput) => Promise<CmsConnection>;
  deleteConnections: (ids: number[]) => Promise<void>;
  testConnection: (id: number) => Promise<{ success: boolean; message: string }>;
  publishArticle: (articleId: number, target: PublishTarget) => Promise<PublishRecord>;
  batchPublish: (articleIds: number[], target: PublishTarget) => Promise<PublishRecord[]>;
}

export const usePublishStore = create<PublishStore>((set, get) => ({
  connections: [],
  records: [],
  loading: false,
  testingId: null,
  publishing: false,

  fetchConnections: async () => {
    set({ loading: true });
    try {
      const connections = await publishService.listCmsConnections();
      set({ connections });
    } finally {
      set({ loading: false });
    }
  },

  fetchRecords: async () => {
    const records = await publishService.listPublishRecords();
    set({ records });
  },

  saveConnection: async (input) => {
    const saved = await publishService.saveCmsConnection(input);
    await get().fetchConnections();
    return saved;
  },

  deleteConnections: async (ids) => {
    await publishService.deleteCmsConnections(ids);
    await get().fetchConnections();
  },

  testConnection: async (id) => {
    set({ testingId: id });
    try {
      return await publishService.testCmsConnection(id);
    } finally {
      set({ testingId: null });
    }
  },

  publishArticle: async (articleId, target) => {
    set({ publishing: true });
    try {
      const record = await publishService.publishArticle(articleId, target);
      await get().fetchRecords();
      return record;
    } finally {
      set({ publishing: false });
    }
  },

  batchPublish: async (articleIds, target) => {
    set({ publishing: true });
    try {
      const records = await publishService.batchPublish(articleIds, target);
      await get().fetchRecords();
      return records;
    } finally {
      set({ publishing: false });
    }
  },
}));
