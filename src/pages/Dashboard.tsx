import React, { useEffect, useState } from 'react';
import { Card, Col, Row, Statistic, List, Tag, Button, Spin, Empty } from 'antd';
import {
  FileTextOutlined,
  CloudUploadOutlined,
  WifiOutlined,
  SearchOutlined,
  RiseOutlined,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { getDashboardStats, type DashboardStats } from '@/services/articleService';
import ArticleCard from '@/components/ArticleCard';

/** 仪表盘页面 */
const Dashboard: React.FC = () => {
  const navigate = useNavigate();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);

  const fetch = async () => {
    setLoading(true);
    try {
      setStats(await getDashboardStats());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void fetch();
  }, []);

  if (loading) {
    return (
      <div className="flex justify-center py-20">
        <Spin size="large" />
      </div>
    );
  }
  if (!stats) return <Empty />;

  return (
    <div className="space-y-4">
      <Row gutter={[16, 16]}>
        <Col span={5}>
          <Card>
            <Statistic
              title="文章总数"
              value={stats.article_total}
              prefix={<FileTextOutlined className="text-maple-500" />}
            />
          </Card>
        </Col>
        <Col span={5}>
          <Card>
            <Statistic
              title="今日采集"
              value={stats.today_collected}
              prefix={<RiseOutlined className="text-emerald-500" />}
              valueStyle={{ color: '#10b981' }}
            />
          </Card>
        </Col>
        <Col span={5}>
          <Card>
            <Statistic
              title="待发布"
              value={stats.pending_publish}
              prefix={<CloudUploadOutlined className="text-blue-500" />}
            />
          </Card>
        </Col>
        <Col span={5}>
          <Card>
            <Statistic title="采集源" value={stats.source_total} prefix={<WifiOutlined className="text-purple-500" />} />
          </Card>
        </Col>
        <Col span={4}>
          <Card>
            <Statistic
              title="关键词任务"
              value={stats.keyword_total}
              prefix={<SearchOutlined className="text-amber-500" />}
            />
          </Card>
        </Col>
      </Row>

      <Row gutter={16}>
        <Col span={14}>
          <Card title="最近采集" extra={<Button type="link" size="small" onClick={() => navigate('/articles')}>查看全部</Button>}>
            {stats.recent_articles.length === 0 ? (
              <Empty description="暂无文章，去关键词采集页试试" image={Empty.PRESENTED_IMAGE_SIMPLE}>
                <Button type="primary" onClick={() => navigate('/keywords')}>
                  开始采集
                </Button>
              </Empty>
            ) : (
              <div className="space-y-2">
                {stats.recent_articles.map((a) => (
                  <ArticleCard key={a.id} article={a} />
                ))}
              </div>
            )}
          </Card>
        </Col>
        <Col span={10}>
          <Card title="发布统计" extra={<Button type="link" size="small" onClick={() => navigate('/publish')}>发布管理</Button>}>
            <div className="mb-4 text-3xl font-bold text-slate-800">
              {stats.published_total}
              <span className="ml-1 text-sm font-normal text-slate-400">篇已发布</span>
            </div>
            <List
              size="small"
              dataSource={stats.publish_by_type}
              renderItem={(item) => (
                <List.Item>
                  <Tag>{item.publish_type}</Tag>
                  <span className="text-slate-600">{item.count} 篇</span>
                </List.Item>
              )}
              locale={{ emptyText: '暂无发布记录' }}
            />
          </Card>
        </Col>
      </Row>
    </div>
  );
};

export default Dashboard;
