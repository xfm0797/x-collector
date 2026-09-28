import { create } from 'zustand';
import type {
  ManualSection,
  ManualContent,
  ConfigExample,
  FaqCategory,
  FaqItem,
  SearchResult,
  QuickStartGuide,
  ShortcutItem,
  ErrorCode,
  UpdateInfo,
} from '@/types/help';
import * as helpService from '@/services/helpService';

interface HelpStore {
  // ── 手册 ──────────────────────────────
  manualIndex: ManualSection[];
  manualContent: ManualContent | null;
  isLoadingManual: boolean;

  // ── 配置示例 ──────────────────────────
  examples: ConfigExample[];

  // ── FAQ ───────────────────────────────
  faqCategories: FaqCategory[];
  faqs: FaqItem[];
  activeFaqCategory: string;

  // ── 搜索 ──────────────────────────────
  searchQuery: string;
  searchResults: SearchResult[];
  isSearching: boolean;

  // ── 其他 ──────────────────────────────
  quickStart: QuickStartGuide | null;
  shortcuts: ShortcutItem[];
  errorCodes: ErrorCode[];
  updateInfo: UpdateInfo | null;

  // ── Actions ───────────────────────────
  fetchManualIndex: () => Promise<void>;
  fetchManualSection: (path: string) => Promise<void>;
  fetchExamples: (platform?: string) => Promise<void>;
  fetchFaqCategories: () => Promise<void>;
  fetchFaqs: (category?: string) => Promise<void>;
  searchHelp: (query: string) => Promise<void>;
  fetchQuickStart: () => Promise<void>;
  fetchShortcuts: () => Promise<void>;
  fetchErrorCodes: () => Promise<void>;
  checkUpdate: () => Promise<void>;
  setSearchQuery: (query: string) => void;
  setActiveFaqCategory: (category: string) => void;
}

export const useHelpStore = create<HelpStore>((set, get) => ({
  manualIndex: [],
  manualContent: null,
  isLoadingManual: false,
  examples: [],
  faqCategories: [],
  faqs: [],
  activeFaqCategory: 'all',
  searchQuery: '',
  searchResults: [],
  isSearching: false,
  quickStart: null,
  shortcuts: [],
  errorCodes: [],
  updateInfo: null,

  fetchManualIndex: async () => {
    const index = await helpService.getManualIndex();
    set({ manualIndex: index });
  },

  fetchManualSection: async (path) => {
    set({ isLoadingManual: true });
    try {
      const content = await helpService.getManualSection(path);
      set({ manualContent: content });
    } finally {
      set({ isLoadingManual: false });
    }
  },

  fetchExamples: async (platform) => {
    const examples = await helpService.getExamples(platform);
    set({ examples });
  },

  fetchFaqCategories: async () => {
    const categories = await helpService.getFaqCategories();
    set({ faqCategories: categories });
  },

  fetchFaqs: async (category) => {
    const faqs = await helpService.getFaqs(category);
    set({ faqs });
  },

  searchHelp: async (query) => {
    if (!query.trim()) {
      set({ searchResults: [] });
      return;
    }
    set({ isSearching: true });
    try {
      const results = await helpService.searchHelp(query);
      set({ searchResults: results });
    } finally {
      set({ isSearching: false });
    }
  },

  fetchQuickStart: async () => {
    if (get().quickStart) return;
    const quickStart = await helpService.getQuickStart();
    set({ quickStart });
  },

  fetchShortcuts: async () => {
    const shortcuts = await helpService.getShortcuts();
    set({ shortcuts });
  },

  fetchErrorCodes: async () => {
    const errorCodes = await helpService.getErrorCodes();
    set({ errorCodes });
  },

  checkUpdate: async () => {
    const updateInfo = await helpService.checkUpdate();
    set({ updateInfo });
  },

  setSearchQuery: (query) => set({ searchQuery: query }),
  setActiveFaqCategory: (category) => set({ activeFaqCategory: category }),
}));
