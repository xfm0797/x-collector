import { useEffect } from 'react';
import { useRewriteStore } from '@/stores/rewriteStore';

/** 伪原创 Hook */
export function useRewrite() {
  const { options, previewData, previewing, fetchOptions, saveOptions, runPreview, applyRewrite, setPreviewData } =
    useRewriteStore();

  useEffect(() => {
    void fetchOptions();
  }, [fetchOptions]);

  return { options, previewData, previewing, fetchOptions, saveOptions, runPreview, applyRewrite, setPreviewData };
}
