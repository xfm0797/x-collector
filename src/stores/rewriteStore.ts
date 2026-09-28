import { create } from 'zustand';
import type { RewriteOptions, RewritePreview } from '@/types/rewrite';
import * as rewriteService from '@/services/rewriteService';
interface RewriteStore {
  options: RewriteOptions | null;
  previewData: RewritePreview | null;
  loading: boolean;
  previewing: boolean;
  fetchOptions: () => Promise<void>;
  saveOptions: (options: RewriteOptions) => Promise<void>;
  setPreviewData: (p: RewritePreview | null) => void;
  runPreview: (articleId: number) => Promise<RewritePreview>;
  applyRewrite: (articleId: number) => Promise<void>;
}

export const useRewriteStore = create<RewriteStore>((set) => ({
  options: null,
  previewData: null,
  loading: false,
  previewing: false,

  fetchOptions: async () => {
    const options = await rewriteService.getRewriteOptions();
    set({ options });
  },

  saveOptions: async (options) => {
    await rewriteService.saveRewriteOptions(options);
    set({ options });
  },

  setPreviewData: (p) => set({ previewData: p }),

  runPreview: async (articleId) => {
    set({ previewing: true });
    try {
      const p = await rewriteService.previewRewrite(articleId);
      set({ previewData: p });
      return p;
    } finally {
      set({ previewing: false });
    }
  },

  applyRewrite: async (articleId) => {
    await rewriteService.applyRewrite(articleId);
  },
}));
