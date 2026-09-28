import React from 'react';
import { Card, Tag } from 'antd';
import { useNavigate } from 'react-router-dom';
import { timeAgo } from '@/utils/date';
import type { Article } from '@/types/article';

interface ArticleCardProps {
  article: Pick<Article, 'id' | 'title' | 'source' | 'collected_at' | 'status'>;
}

const statusColor: Record<string, string> = {
  collected: 'default',
  edited: 'blue',
  published: 'green',
  failed: 'red',
};

const statusText: Record<string, string> = {
  collected: '已采集',
  edited: '已编辑',
  published: '已发布',
  failed: '失败',
};

/** 文章卡片：用于仪表盘最近采集列表 */
const ArticleCard: React.FC<ArticleCardProps> = ({ article }) => {
  const navigate = useNavigate();

  return (
    <Card
      size="small"
      hoverable
      className="cursor-pointer"
      onClick={() => navigate(`/articles/${article.id}`)}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="truncate font-medium text-slate-800">{article.title}</span>
        <Tag color={statusColor[article.status]}>{statusText[article.status] || article.status}</Tag>
      </div>
      <div className="mt-1 flex items-center gap-2 text-xs text-slate-400">
        <span>{article.source || '未知来源'}</span>
        <span>·</span>
        <span>{timeAgo(article.collected_at)}</span>
      </div>
    </Card>
  );
};

export default ArticleCard;
