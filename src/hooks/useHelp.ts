import { useEffect } from 'react';
import { useHelpStore } from '@/stores/helpStore';

/** 帮助系统 Hook */
export function useHelp() {
  const store = useHelpStore();

  useEffect(() => {
    if (store.manualIndex.length === 0) {
      void store.fetchManualIndex();
    }
    if (store.faqCategories.length === 0) {
      void store.fetchFaqCategories();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return store;
}
