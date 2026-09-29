import React, { useEffect, useState } from 'react';
import { Card, Table, Button, Space, Tag, Switch, Popconfirm, message, Modal, Form, Input, Select } from 'antd';
import { PlusOutlined, DeleteOutlined, SyncOutlined, RadarChartOutlined, AppstoreOutlined, AimOutlined } from '@ant-design/icons';
import ProgressBar from '@/components/ProgressBar';
import VisualSelector from '@/components/VisualSelector';
import TemplateLibrary from '@/components/TemplateLibrary';
import { useSourceStore } from '@/stores/sourceStore';
import { useCollectProgress } from '@/hooks/useCollect';
import { discoverFromSitemap } from '@/services/collectService';
import type { CollectSource, CollectSourceInput } from '@/types/keyword';
import { timeAgo } from '@/utils/date';

const typeText: Record<string, string> = {
  rss: 'RSS 订阅',
  webpage: '网页采集',
  sitemap: '站点地图',
  api: 'API 接口',
};

const typeColor: Record<string, string> = {
  rss: 'orange',
  webpage: 'blue',
  sitemap: 'purple',
  api: 'cyan',
};

/** 采集源管理页面 */
const SourceManager: React.FC = () => {
  const { sources, loading, collectingSourceId, fetchSources, createSource, updateSource, deleteSources, collectFromSource } =
    useSourceStore();
  const { progress } = useCollectProgress();
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<CollectSource | null>(null);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [sitemapUrl, setSitemapUrl] = useState('');
  const [sitemapOpen, setSitemapOpen] = useState(false);
  const [discovering, setDiscovering] = useState(false);
  const [discovered, setDiscovered] = useState<string[]>([]);
  const [templateOpen, setTemplateOpen] = useState(false);
  /** 可视化选择器状态：目标表单字段名 */
  const [visualTarget, setVisualTarget] = useState<string | null>(null);
  const [form] = Form.useForm();

  useEffect(() => {
    void fetchSources();
  }, [fetchSources]);

  const handleCollect = async (source: CollectSource) => {
    try {
      const summary = await collectFromSource(source.id);
      message.success(`采集完成：发现 ${summary.found} 条，入库 ${summary.collected} 篇，跳过 ${summary.skipped}`);
    } catch (e) {
      message.error(`采集失败：${String(e)}`);
    }
  };

  const handleSave = async (values: CollectSourceInput) => {
    if (editing) {
      await updateSource(editing.id, values);
      message.success('已更新');
    } else {
      await createSource(values);
      message.success('已添加');
    }
  };

  const handleDiscover = async () => {
    if (!sitemapUrl.trim()) return;
    setDiscovering(true);
    try {
      const urls = await discoverFromSitemap(sitemapUrl.trim());
      setDiscovered(urls);
      message.info(`发现 ${urls.length} 个链接，可将其添加为网页采集源`);
    } catch (e) {
      message.error(String(e));
    } finally {
      setDiscovering(false);
    }
  };

  const columns = [
    { title: '名称', dataIndex: 'name', ellipsis: true, render: (v: string) => <span className="font-medium">{v}</span> },
    { title: 'URL', dataIndex: 'url', ellipsis: true, render: (v: string) => <span className="text-xs text-slate-500">{v}</span> },
    {
      title: '类型',
      dataIndex: 'source_type',
      width: 100,
      render: (v: string) => <Tag color={typeColor[v]}>{typeText[v] || v}</Tag>,
    },
    { title: '分组', dataIndex: 'group_name', width: 90, ellipsis: true },
    {
      title: '最后采集',
      dataIndex: 'last_collected',
      width: 110,
      render: (v?: string) => timeAgo(v),
    },
    {
      title: '启用',
      dataIndex: 'enabled',
      width: 70,
      render: (v: number, record: CollectSource) => (
        <Switch size="small" checked={v === 1} onChange={(checked) => updateSource(record.id, { enabled: checked ? 1 : 0 })} />
      ),
    },
    {
      title: '操作',
      width: 200,
      render: (_: unknown, record: CollectSource) => (
        <Space>
          <Button
            type="link"
            size="small"
            icon={<SyncOutlined />}
            loading={collectingSourceId === record.id}
            onClick={() => handleCollect(record)}
          >
            采集
          </Button>
          <Button
            type="link"
            size="small"
            onClick={() => {
              setEditing(record);
              form.resetFields();
              form.setFieldsValue(record);
              setFormOpen(true);
            }}
          >
            编辑
          </Button>
          <Popconfirm title="确认删除该采集源？" onConfirm={() => deleteSources([record.id])}>
            <Button type="link" size="small" danger icon={<DeleteOutlined />} />
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div className="space-y-3">
      <Card size="small">
        <div className="flex items-center justify-between">
          <span className="text-base font-semibold">采集源管理</span>
          <Space>
            {selectedIds.length > 0 && (
              <Popconfirm title={`确认删除选中的 ${selectedIds.length} 个采集源？`} onConfirm={() => deleteSources(selectedIds)}>
                <Button danger icon={<DeleteOutlined />}>
                  批量删除（{selectedIds.length}）
                </Button>
              </Popconfirm>
            )}
            <Button icon={<RadarChartOutlined />} onClick={() => setSitemapOpen(true)}>
              从站点地图发现
            </Button>
            <Button icon={<AppstoreOutlined />} onClick={() => setTemplateOpen(true)}>
              模板库
            </Button>
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => {
                setEditing(null);
                form.resetFields();
                form.setFieldsValue({ source_type: 'rss', group_name: '默认', enabled: true });
                setFormOpen(true);
              }}
            >
              添加采集源
            </Button>
          </Space>
        </div>
      </Card>

      {progress && <ProgressBar progress={progress} />}

      <Card size="small">
        <Table
          rowKey="id"
          columns={columns}
          dataSource={sources}
          loading={loading}
          pagination={false}
          rowSelection={{
            selectedRowKeys: selectedIds,
            onChange: (keys) => setSelectedIds(keys as number[]),
          }}
          locale={{ emptyText: '暂无采集源' }}
        />
      </Card>

      <Modal
        title={editing ? '编辑采集源' : '添加采集源'}
        open={formOpen}
        onCancel={() => setFormOpen(false)}
        onOk={async () => {
          const values = await form.validateFields();
          await handleSave({
            ...values,
            remove_selectors: values.remove_selectors
              ? String(values.remove_selectors)
                  .split(/[\n,，]+/)
                  .map((s: string) => s.trim())
                  .filter(Boolean)
              : undefined,
          });
          setFormOpen(false);
        }}
        destroyOnClose
      >
        <Form form={form} layout="vertical">
          <Form.Item name="name" label="名称" rules={[{ required: true, message: '请输入名称' }]}>
            <Input placeholder="例如：阮一峰周刊" />
          </Form.Item>
          <Form.Item name="url" label="URL" rules={[{ required: true, message: '请输入 URL' }]}>
            <Input placeholder="https://example.com/feed.xml" />
          </Form.Item>
          <div className="grid grid-cols-2 gap-3">
            <Form.Item name="source_type" label="类型">
              <Select
                options={[
                  { value: 'rss', label: 'RSS 订阅' },
                  { value: 'webpage', label: '网页采集' },
                  { value: 'sitemap', label: '站点地图' },
                  { value: 'api', label: 'API 接口' },
                ]}
              />
            </Form.Item>
            <Form.Item name="group_name" label="分组">
              <Input placeholder="默认" />
            </Form.Item>
          </div>
          <Form.Item
            name="selector_content"
            label={
              <span className="flex items-center gap-2">
                正文选择器（可选）
                <Button
                  type="link"
                  size="small"
                  icon={<AimOutlined />}
                  className="!p-0"
                  onClick={() => setVisualTarget('selector_content')}
                >
                  可视化选择
                </Button>
              </span>
            }
            tooltip="CSS 选择器，留空使用智能提取"
          >
            <Input placeholder="例如：div.article-content" />
          </Form.Item>
          <Form.Item
            name="selector_title"
            label={
              <span className="flex items-center gap-2">
                标题选择器（可选）
                <Button
                  type="link"
                  size="small"
                  icon={<AimOutlined />}
                  className="!p-0"
                  onClick={() => setVisualTarget('selector_title')}
                >
                  可视化选择
                </Button>
              </span>
            }
          >
            <Input placeholder="例如：h1.title" />
          </Form.Item>
          <Form.Item
            name="remove_selectors"
            label={
              <span className="flex items-center gap-2">
                移除元素选择器（逗号或换行分隔，可选）
                <Button
                  type="link"
                  size="small"
                  icon={<AimOutlined />}
                  className="!p-0"
                  onClick={() => setVisualTarget('remove_selectors')}
                >
                  可视化选择
                </Button>
              </span>
            }
          >
            <Input.TextArea rows={2} placeholder="例如：div.ad, div.related-posts" />
          </Form.Item>
          <Form.Item name="enabled" label="启用" valuePropName="checked">
            <Switch />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title="从站点地图发现链接"
        open={sitemapOpen}
        onCancel={() => setSitemapOpen(false)}
        onOk={handleDiscover}
        okText="开始发现"
        confirmLoading={discovering}
        width={640}
      >
        <Input
          placeholder="https://example.com/sitemap.xml"
          value={sitemapUrl}
          onChange={(e) => setSitemapUrl(e.target.value)}
        />
        {discovered.length > 0 && (
          <div className="mt-3 max-h-72 overflow-auto rounded border border-slate-200 p-2">
            {discovered.map((u) => (
              <div key={u} className="truncate py-0.5 text-xs text-slate-600">
                {u}
              </div>
            ))}
          </div>
        )}
      </Modal>

      {/* 可视化选择器 */}
      <VisualSelector
        open={visualTarget !== null}
        onClose={() => setVisualTarget(null)}
        initialUrl={form.getFieldValue('url') || ''}
        onPick={(selector) => {
          if (!visualTarget) return;
          if (visualTarget === 'remove_selectors') {
            // 追加到已有移除列表
            const existing = String(form.getFieldValue('remove_selectors') || '').trim();
            const merged = existing ? `${existing}, ${selector}` : selector;
            form.setFieldValue('remove_selectors', merged);
          } else {
            form.setFieldValue(visualTarget, selector);
          }
        }}
      />

      {/* 模板库 */}
      <TemplateLibrary
        open={templateOpen}
        mode="source"
        onClose={() => setTemplateOpen(false)}
        onApplySource={(values) => {
          const { remove_selectors, ...rest } = values;
          form.setFieldsValue({
            ...rest,
            ...(remove_selectors ? { remove_selectors: remove_selectors.join(', ') } : {}),
          });
        }}
        onApplyCleaning={(selectors) => {
          const existing = String(form.getFieldValue('remove_selectors') || '').trim();
          form.setFieldValue(
            'remove_selectors',
            existing ? [...new Set([...existing.split(/[\n,，]+/).map((s) => s.trim()).filter(Boolean), ...selectors])].join(', ') : selectors.join(', '),
          );
        }}
      />
    </div>
  );
};

export default SourceManager;
