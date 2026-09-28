import React from 'react';
import { Progress } from 'antd';
import type { CollectProgress } from '@/types/common';

interface ProgressBarProps {
  progress: CollectProgress | null;
}

/** 采集进度条 */
const ProgressBar: React.FC<ProgressBarProps> = ({ progress }) => {
  if (!progress) return null;
  const percent = progress.total > 0 ? Math.round((progress.current / progress.total) * 100) : 0;
  return (
    <div className="rounded-lg border border-slate-200 bg-white p-4">
      <div className="mb-2 flex items-center justify-between text-sm">
        <span className="font-medium text-slate-700">{progress.stage}</span>
        <span className="text-slate-500">
          {progress.current}/{progress.total}（{percent}%）
        </span>
      </div>
      <Progress percent={percent} showInfo={false} strokeColor="#f97316" />
      <div className="mt-1 truncate text-xs text-slate-500">{progress.message}</div>
    </div>
  );
};

export default ProgressBar;
