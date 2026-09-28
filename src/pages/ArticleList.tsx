import React, { useEffect, useState } from 'react';
import { Table, Card, Input, Select, Button, Space, Tag, Popconfirm, message, Modal } from 'antd';
import { PlusOutlined, DeleteOutlined, LinkOutlined, SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import type { Article } from '@/types/article';
import { useArticles } from '@/hooks/useArticles';
import { extractExcerpt } from '@/utils/markdown';
import { formatDate } from '@/utils/date';

const statusOptions = [
  { value: '', label: '全部状态' },
  { value: 'collected', label: '已采集' },
  { value: 'edited', label: '已编辑' },
  { value: 'published', label: '已发布' },
  { value: 'failed', label: '发布失败' },
];

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

/** 文章列表页面 */
const ArticleList: React.FC = () => {
  const navigate = useNavigate();
  const { articles, total, page, pageSize, loading, setQuery, refresh, deleteArticles } = useArticles();
  const [keyword, setKeyword] = useState('');
  const [status, setStatus] = useState('');
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [collectUrl, setCollectUrl] = useState('');
  const [collecting, setCollecting] = useState(false);
  const [collectOpen, setCollectOpen] = useState(false);

  useEffect(() => {
    setSelectedIds([]);
  }, [articles]);

  const handleSearch = () => {
    setQuery({ keyword: keyword || undefined, status: status || undefined, page: 1 });
  };

  const handleDelete = async (ids: number[]) => {
    await deleteArticles(ids);
    message.success(`已删除 ${ids.length} 篇文章`);
  };

  const handleCollect = async () => {
    if (!collectUrl.trim()) return;
    setCollecting(true);
    try {
      const { useArticleStore } = await import('@/stores/articleStore');
      const article = await useArticleStore.getState().collectSinglePage(collectUrl.trim());
      message.success(`采集成功：${article.title}`);
      setCollectOpen(false);
      setCollectUrl('');
      refresh();
    } catch (e) {
      message.error(`采集失败：${String(e)}`);
    } finally {
      setCollecting(false);
    }
  };

  const columns = [
    {
      title: '标题',
      dataIndex: 'title',
      ellipsis: true,
      render: (title: string, record: Article) => (
        <a onClick={() => navigate(`/articles/${record.id}`)} className="font-medium">
          {title}
        </a>
      ),
    },
    {
      title: '摘要',
      dataIndex: 'content_md',
      ellipsis: true,
      width: '30%',
      render: (md?: string) => <span className="text-slate-500">{md ? extractExcerpt(md, 80) : '-'}</span>,
    },
    {
      title: '来源',
      dataIndex: 'source',
      width: 120,
      ellipsis: true,
      render: (source?: string) => source || '-',
    },
    {
      title: '状态',
      dataIndex: 'status',
      width: 100,
      render: (s: string) => <Tag color={statusColor[s]}>{statusText[s] || s}</Tag>,
    },
    {
      title: '采集时间',
      dataIndex: 'collected_at',
      width: 120,
      render: (v?: string) => formatDate(v),
    },
    {
      title: '操作',
      width: 140,
      render: (_: unknown, record: Article) => (
        <Space>
          <Button type="link" size="small" icon={<LinkOutlined />} href={record.url} target="_blank">
            原文
          </Button>
          <Popconfirm title="确认删除该文章？" onConfirm={() => handleDelete([record.id])}>
            <Button type="link" size="small" danger icon={<DeleteOutlined />}>
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div className="space-y-3">
      <Card size="small">
        <div className="flex items-center justify-between">
          <Space>
            <Input
              allowClear
              prefix={<SearchOutlined />}
              placeholder="按标题/内容/标签搜索"
              style={{ width: 260 }}
              value={keyword}
              onChange={(e) => setKeyword(e.target.value)}
              onPressEnter={handleSearch}
            />
            <Select
              style={{ width: 130 }}
              value={status}
              options={statusOptions}
              onChange={(v) => {
                setStatus(v);
                setQuery({ status: v || undefined, page: 1 });
              }}
            />
            <Button type="primary" onClick={handleSearch}>
              搜索
            </Button>
          </Space>
          <Space>
            {selectedIds.length > 0 && (
              <Popconfirm title={`确认删除选中的 ${selectedIds.length} 篇文章？`} onConfirm={() => handleDelete(selectedIds)}>
                <Button danger icon={<DeleteOutlined />}>
                  批量删除（{selectedIds.length}）
                </Button>
              </Popconfirm>
            )}
            <Button icon={<LinkOutlined />} onClick={() => setCollectOpen(true)}>
              单页采集
            </Button>
            <Button type="primary" icon={<PlusOutlined />} onClick={() => navigate('/articles/new')}>
              新建文章
            </Button>
          </Space>
        </div>
      </Card>

      <Card size="small">
        <Table
          rowKey="id"
          columns={columns}
          dataSource={articles}
          loading={loading}
          rowSelection={{
            selectedRowKeys: selectedIds,
            onChange: (keys) => setSelectedIds(keys as number[]),
          }}
          pagination={{
            current: page,
            pageSize,
            total,
            showTotal: (t) => `共 ${t} 篇`,
            onChange: (p) => setQuery({ page: p }),
          }}
        />
      </Card>

      <Modal
        title="单页采集"
        open={collectOpen}
        onCancel={() => setCollectOpen(false)}
        onOk={handleCollect}
        okText="开始采集"
        confirmLoading={collecting}
      >
        <Input
          placeholder="输入文章 URL，例如 https://example.com/post/123"
          value={collectUrl}
          onChange={(e) => setCollectUrl(e.target.value)}
        />
      </Modal>
    </div>
  );
};

export default ArticleList;
