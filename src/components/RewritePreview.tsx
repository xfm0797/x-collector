import React from 'react';
import { Card, Tag, Empty, Spin } from 'antd';
import MarkdownRenderer from './MarkdownRenderer';
import type { RewritePreview } from '@/types/rewrite';

interface RewritePreviewProps {
  preview: RewritePreview | null;
  loading?: boolean;
}

/** 伪原创预览：原文 vs 改写后 双栏对比 */
const RewritePreview: React.FC<RewritePreviewProps> = ({ preview, loading }) => {
  if (loading) {
    return (
      <div className="flex justify-center py-10">
        <Spin tip="正在改写..." />
      </div>
    );
  }
  if (!preview) {
    return <Empty description="点击「预览改写」查看对比效果" />;
  }

  const diffStats = () => {
    const a = preview.original;
    const b = preview.rewritten;
    let same = 0;
    const minLen = Math.min(a.length, b.length);
    for (let i = 0; i < minLen; i++) if (a[i] === b[i]) same++;
    return minLen > 0 ? Math.round((1 - same / minLen) * 100) : 0;
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2 text-sm">
        <Tag color="orange">改写率约 {diffStats()}%</Tag>
        <Tag>{preview.options.intensity === 'light' ? '轻度' : preview.options.intensity === 'medium' ? '中等' : '深度'}</Tag>
        <Tag color="blue">同义词 {preview.options.synonym_ratio}%</Tag>
        <Tag color="green">句子 {preview.options.sentence_ratio}%</Tag>
        {preview.options.paragraph_shuffle && <Tag color="purple">段落重排</Tag>}
        {preview.options.rewrite_ends && <Tag color="cyan">首尾重写</Tag>}
      </div>
      <div className="grid grid-cols-2 gap-3">
        <Card size="small" title="原文" className="overflow-auto" style={{ maxHeight: 480 }} styles={{ body: { maxHeight: 400, overflow: 'auto' } }}>
          <MarkdownRenderer content={preview.original} />
        </Card>
        <Card size="small" title="改写后" className="overflow-auto" style={{ maxHeight: 480 }} styles={{ body: { maxHeight: 400, overflow: 'auto' } }}>
          <MarkdownRenderer content={preview.rewritten} />
        </Card>
      </div>
    </div>
  );
};

export default RewritePreview;
