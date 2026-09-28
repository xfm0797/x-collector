import React, { useEffect, useState } from 'react';
import { Card, Table, Button, Space, Tag, Popconfirm, message, Tabs, List, Typography } from 'antd';
import { PlusOutlined, DeleteOutlined, ApiOutlined, CloudUploadOutlined, FileTextOutlined } from '@ant-design/icons';
import CmsConfigForm from '@/components/CmsConfigForm';
import PublishDialog from '@/components/PublishDialog';
import { usePublish } from '@/hooks/usePublish';
import { useArticleStore } from '@/stores/articleStore';
import type { CmsConnection, CmsConnectionInput, PublishRecord } from '@/types/publish';
import { formatDateTime } from '@/utils/date';

const cmsText: Record<string, string> = {
  wordpress: 'WordPress',
  typecho: 'Typecho',
  zblog: 'Z-Blog',
  custom: '自定义',
};

/** 发布管理页面：CMS 连接管理 + 发布记录 + 待发布文章 */
const PublishManager: React.FC = () => {
  const {
    connections,
    records,
    loading,
    testingId,
    publishing,
    saveConnection,
    deleteConnections,
    testConnection,
    batchPublish,
    fetchRecords,
  } = usePublish();
  const { articles, fetchArticles } = useArticleStore();
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<CmsConnection | null>(null);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [publishOpen, setPublishOpen] = useState(false);
  const [tab, setTab] = useState<'connections' | 'pending' | 'records'>('connections');

  useEffect(() => {
    void fetchArticles();
  }, [fetchArticles]);

  const handleTest = async (conn: CmsConnection) => {
    const result = await testConnection(conn.id);
    if (result.success) {
      message.success(result.message);
    } else {
      message.error(result.message);
    }
  };

  const handlePublish = async (target: { publish_type: string; cms_id?: number; output_dir?: string }) => {
    try {
      const results = await batchPublish(selectedIds, target as never);
      const ok = results.filter((r) => r.status === 'success').length;
      const fail = results.length - ok;
      if (fail === 0) message.success(`成功发布 ${ok} 篇文章`);
      else message.warning(`发布完成：成功 ${ok} 篇，失败 ${fail} 篇`);
      void fetchRecords();
      void fetchArticles();
    } catch (e) {
      message.error(String(e));
    }
  };

  const pendingArticles = articles.filter((a) => a.status === 'collected' || a.status === 'edited');

  const connectionColumns = [
    { title: '名称', dataIndex: 'name', render: (v: string) => <span className="font-medium">{v}</span> },
    {
      title: '类型',
      dataIndex: 'cms_type',
      width: 110,
      render: (v: string) => <Tag color="geekblue">{cmsText[v] || v}</Tag>,
    },
    { title: '站点地址', dataIndex: 'site_url', ellipsis: true, render: (v: string) => <span className="text-xs">{v}</span> },
    { title: '用户名', dataIndex: 'username', width: 110, ellipsis: true },
    {
      title: '状态',
      dataIndex: 'enabled',
      width: 70,
      render: (v: number) => (v === 1 ? <Tag color="green">启用</Tag> : <Tag>禁用</Tag>),
    },
    {
      title: '操作',
      width: 220,
      render: (_: unknown, record: CmsConnection) => (
        <Space>
          <Button
            type="link"
            size="small"
            icon={<ApiOutlined />}
            loading={testingId === record.id}
            onClick={() => handleTest(record)}
          >
            测试
          </Button>
          <Button
            type="link"
            size="small"
            onClick={() => {
              setEditing(record);
              setFormOpen(true);
            }}
          >
            编辑
          </Button>
          <Popconfirm title="确认删除该连接？" onConfirm={() => deleteConnections([record.id])}>
            <Button type="link" size="small" danger icon={<DeleteOutlined />} />
          </Popconfirm>
        </Space>
      ),
    },
  ];

  const statusColor: Record<string, string> = { success: 'green', failed: 'red', pending: 'gold' };
  const statusText: Record<string, string> = { success: '成功', failed: '失败', pending: '待发布' };

  const recordColumns = [
    {
      title: '文章',
      dataIndex: 'article_title',
      ellipsis: true,
      render: (v?: string) => v || '-',
    },
    {
      title: '发布方式',
      dataIndex: 'publish_type',
      width: 100,
      render: (v: string) => <Tag>{v === 'hexo' ? 'Hexo 导出' : cmsText[v] || v}</Tag>,
    },
    {
      title: '状态',
      dataIndex: 'status',
      width: 80,
      render: (v: string, record: PublishRecord) => (
        <TooltipRecordStatus status={v} record={record} />
      ),
    },
    {
      title: '远程地址',
      dataIndex: 'remote_url',
      ellipsis: true,
      render: (v?: string) =>
        v ? (
          <Typography.Link href={v} target="_blank" className="text-xs">
            {v}
          </Typography.Link>
        ) : (
          '-'
        ),
    },
    { title: '发布时间', dataIndex: 'published_at', width: 150, render: (v?: string) => formatDateTime(v) },
  ];

  const TooltipRecordStatus: React.FC<{ status: string; record: PublishRecord }> = ({ status, record }) => (
    <Tag color={statusColor[status]}>
      {statusText[status] || status}
      {status === 'failed' && record.error_message ? ' ⚠' : ''}
    </Tag>
  );

  return (
    <div className="space-y-3">
      <Card size="small">
        <Tabs
          activeKey={tab}
          onChange={(k) => setTab(k as typeof tab)}
          items={[
            {
              key: 'connections',
              label: (
                <span>
                  <ApiOutlined /> CMS 连接
                </span>
              ),
              children: (
                <>
                  <div className="mb-3 flex justify-end">
                    <Button
                      type="primary"
                      icon={<PlusOutlined />}
                      onClick={() => {
                        setEditing(null);
                        setFormOpen(true);
                      }}
                    >
                      添加连接
                    </Button>
                  </div>
                  <Table
                    rowKey="id"
                    columns={connectionColumns}
                    dataSource={connections}
                    loading={loading}
                    pagination={false}
                    locale={{ emptyText: '暂无 CMS 连接' }}
                  />
                </>
              ),
            },
            {
              key: 'pending',
              label: (
                <span>
                  <FileTextOutlined /> 待发布文章（{pendingArticles.length}）
                </span>
              ),
              children: (
                <>
                  <div className="mb-3 flex items-center justify-between">
                    <span className="text-sm text-slate-500">选择文章后批量发布到 Hexo 或 CMS</span>
                    <Button
                      type="primary"
                      icon={<CloudUploadOutlined />}
                      disabled={selectedIds.length === 0}
                      loading={publishing}
                      onClick={() => setPublishOpen(true)}
                    >
                      批量发布（{selectedIds.length}）
                    </Button>
                  </div>
                  <List
                    size="small"
                    bordered
                    dataSource={pendingArticles}
                    locale={{ emptyText: '暂无待发布文章' }}
                    renderItem={(a) => (
                      <List.Item
                        className="cursor-pointer"
                        onClick={() =>
                          setSelectedIds((prev) =>
                            prev.includes(a.id) ? prev.filter((i) => i !== a.id) : [...prev, a.id],
                          )
                        }
                      >
                        <Tag color={selectedIds.includes(a.id) ? 'orange' : 'default'}>
                          {selectedIds.includes(a.id) ? '已选' : '未选'}
                        </Tag>
                        <span className="flex-1 truncate">{a.title}</span>
                        <span className="text-xs text-slate-400">{formatDateTime(a.collected_at)}</span>
                      </List.Item>
                    )}
                  />
                </>
              ),
            },
            {
              key: 'records',
              label: '发布记录',
              children: (
                <Table
                  rowKey="id"
                  columns={recordColumns}
                  dataSource={records}
                  pagination={{ pageSize: 10, showTotal: (t) => `共 ${t} 条` }}
                  locale={{ emptyText: '暂无发布记录' }}
                />
              ),
            },
          ]}
        />
      </Card>

      <CmsConfigForm
        open={formOpen}
        initial={editing}
        onCancel={() => setFormOpen(false)}
        onSubmit={(input: CmsConnectionInput) => saveConnection(input)}
      />

      <PublishDialog
        open={publishOpen}
        articleIds={selectedIds}
        connections={connections}
        onCancel={() => setPublishOpen(false)}
        onSubmit={handlePublish}
      />
    </div>
  );
};

export default PublishManager;
