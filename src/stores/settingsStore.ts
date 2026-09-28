import { create } from 'zustand';
import * as settingsService from '@/services/settingsService';

interface SettingsStore {
  settings: Record<string, string>;
  loading: boolean;
  saving: boolean;
  fetchSettings: () => Promise<void>;
  saveSettings: (settings: Record<string, string>) => Promise<void>;
  setValue: (key: string, value: string) => void;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: {},
  loading: false,
  saving: false,

  fetchSettings: async () => {
    set({ loading: true });
    try {
      const settings = await settingsService.getSettings();
      set({ settings });
    } finally {
      set({ loading: false });
    }
  },

  saveSettings: async (settings) => {
    set({ saving: true });
    try {
      await settingsService.saveSettings(settings);
      set({ settings: { ...get().settings, ...settings } });
    } finally {
      set({ saving: false });
    }
  },

  setValue: (key, value) => set({ settings: { ...get().settings, [key]: value } }),
}));
