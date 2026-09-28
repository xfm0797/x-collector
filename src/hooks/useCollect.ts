import { useEffect, useRef, useState } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { CollectProgress, CollectLogEntry } from '@/types/common';

/** 采集进度 Hook：订阅 Rust 端发出的采集进度与日志事件 */
export function useCollectProgress() {
  const [progress, setProgress] = useState<CollectProgress | null>(null);
  const [logs, setLogs] = useState<CollectLogEntry[]>([]);
  const unlistenRef = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    let mounted = true;
    const p1 = listen<CollectProgress>('collect:progress', (e) => {
      if (mounted) setProgress(e.payload);
    });
    const p2 = listen<CollectLogEntry>('collect:log', (e) => {
      if (mounted) setLogs((prev) => [...prev.slice(-200), e.payload]);
    });
    Promise.all([p1, p2]).then((fns) => {
      unlistenRef.current = fns;
    });
    return () => {
      mounted = false;
      unlistenRef.current.forEach((fn) => fn());
      unlistenRef.current = [];
    };
  }, []);

  const clear = () => {
    setProgress(null);
    setLogs([]);
  };

  return { progress, logs, clear };
}
