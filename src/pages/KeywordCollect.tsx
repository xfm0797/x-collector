import React, { useEffect, useState } from 'react';
import { Card, Table, Button, Space, Tag, Switch, Popconfirm, message, Tabs, List, Tooltip } from 'antd';
import { PlusOutlined, DeleteOutlined, PlayCircleOutlined } from '@ant-design/icons';
import KeywordForm from '@/components/KeywordForm';
import ProgressBar from '@/components/ProgressBar';
import { useKeywords } from '@/hooks/useKeywords';
import { useCollectProgress } from '@/hooks/useCollect';
import { listCollectLogs, type CollectLog } from '@/services/keywordService';
import { useArticleStore } from '@/stores/articleStore';
import type { KeywordTask, KeywordTaskInput } from '@/types/keyword';
import { formatDateTime, timeAgo } from '@/utils/date';

const engineText: Record<string, string> = {
  baidu: '百度',
  google: 'Google',
  bing: 'Bing',
  sogou: '搜狗',
  custom: '自定义',
};

const matchText: Record<string, string> = {
  title: '标题匹配',
  content: '全文匹配',
  title_or_content: '标题或全文',
  exact: '精确匹配',
};

/** 关键词采集页面 */
const KeywordCollect: React.FC = () => {
  const { tasks, loading, runningTaskId, lastSummary, createTask, updateTask, deleteTasks, runTask } = useKeywords();
  const { progress, logs: _logs, clear } = useCollectProgress();
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Partial<KeywordTaskInput> | undefined>();
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [logTab, setLogTab] = useState<'progress' | 'logs'>('progress');
  const [logs, setLogs] = useState<CollectLog[]>([]);

  const refreshLogs = async () => {
    try {
      setLogs(await listCollectLogs(50));
    } catch {
      // 忽略日志加载失败
    }
  };

  useEffect(() => {
    void refreshLogs();
  }, []);

  const handleRun = async (task: KeywordTask) => {
    clear();
    setLogTab('progress');
    try {
      const summary = await runTask(task.id);
      message.success(
        `采集完成：发现 ${summary.found} 条，入库 ${summary.collected} 篇，去重跳过 ${summary.skipped}，失败 ${summary.failed}`,
      );
      void refreshLogs();
      void useArticleStore.getState().fetchArticles();
    } catch (e) {
      message.error(`采集失败：${String(e)}`);
    }
  };

  const handleToggle = async (task: KeywordTask, enabled: boolean) => {
    await updateTask(task.id, { enabled: enabled ? 1 : 0 });
  };

  const handleSubmit = async (input: KeywordTaskInput) => {
    if (editing?.keyword) {
      await updateTask((editing as KeywordTask).id, input);
      message.success('已更新');
    } else {
      await createTask(input);
      message.success('已添加');
    }
  };

  const columns = [
    { title: '关键词', dataIndex: 'keyword', ellipsis: true, render: (v: string) => <span className="font-medium">{v}</span> },
    { title: '分组', dataIndex: 'group_name', width: 100, ellipsis: true },
    {
      title: '搜索引擎',
      dataIndex: 'search_engine',
      width: 100,
      render: (v: string) => <Tag color="orange">{engineText[v] || v}</Tag>,
    },
    { title: '站点限定', dataIndex: 'site_limit', width: 120, ellipsis: true, render: (v?: string) => v || '-' },
    { title: '匹配模式', dataIndex: 'match_mode', width: 110, render: (v: string) => matchText[v] || v },
    { title: '深度', dataIndex: 'max_pages', width: 60, render: (v: number) => `${v} 页` },
    {
      title: '轮询',
      dataIndex: 'interval_minutes',
      width: 90,
      render: (v: number) => (v > 0 ? `${v} 分钟` : '手动'),
    },
    {
      title: '上次运行',
      dataIndex: 'last_run',
      width: 110,
      render: (v?: string) => <Tooltip title={formatDateTime(v)}>{timeAgo(v)}</Tooltip>,
    },
    {
      title: '启用',
      dataIndex: 'enabled',
      width: 70,
      render: (v: number, record: KeywordTask) => (
        <Switch size="small" checked={v === 1} onChange={(checked) => handleToggle(record, checked)} />
      ),
    },
    {
      title: '操作',
      width: 190,
      render: (_: unknown, record: KeywordTask) => (
        <Space>
          <Button
            type="link"
            size="small"
            icon={<PlayCircleOutlined />}
            loading={runningTaskId === record.id}
            onClick={() => handleRun(record)}
          >
            采集
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
          <Popconfirm title="确认删除该任务？" onConfirm={() => deleteTasks([record.id])}>
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
          <span className="text-base font-semibold">关键词采集任务</span>
          <Space>
            {selectedIds.length > 0 && (
              <Popconfirm title={`确认删除选中的 ${selectedIds.length} 个任务？`} onConfirm={() => deleteTasks(selectedIds)}>
                <Button danger icon={<DeleteOutlined />}>
                  批量删除（{selectedIds.length}）
                </Button>
              </Popconfirm>
            )}
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => {
                setEditing(undefined);
                setFormOpen(true);
              }}
            >
              添加关键词
            </Button>
          </Space>
        </div>
      </Card>

      <Card size="small">
        <Table
          rowKey="id"
          columns={columns}
          dataSource={tasks}
          loading={loading}
          pagination={false}
          rowSelection={{
            selectedRowKeys: selectedIds,
            onChange: (keys) => setSelectedIds(keys as number[]),
          }}
          locale={{ emptyText: '暂无关键词任务，点击右上角添加' }}
        />
      </Card>

      <Card size="small">
        <Tabs
          activeKey={logTab}
          onChange={(k) => setLogTab(k as 'progress' | 'logs')}
          items={[
            {
              key: 'progress',
              label: '采集进度',
              children: progress ? <ProgressBar progress={progress} /> : <span className="text-sm text-slate-400">暂无进行中的采集任务</span>,
            },
            {
              key: 'logs',
              label: '采集日志',
              children: (
                <List
                  size="small"
                  dataSource={logs}
                  renderItem={(log) => (
                    <List.Item className="text-sm">
                      <Tag color={log.status === 'success' ? 'green' : log.status === 'failed' ? 'red' : 'blue'}>
                        {log.status === 'success' ? '成功' : log.status === 'failed' ? '失败' : '跳过'}
                      </Tag>
                      <span className="flex-1 truncate">{log.keyword || log.url || log.message || '-'}</span>
                      <span className="ml-2 shrink-0 text-xs text-slate-400">{formatDateTime(log.created_at)}</span>
                    </List.Item>
                  )}
                  locale={{ emptyText: '暂无采集日志' }}
                />
              ),
            },
          ]}
        />
        {lastSummary && (
          <div className="mt-2 text-xs text-slate-500">
            上次结果：发现 {lastSummary.found} / 入库 {lastSummary.collected} / 跳过 {lastSummary.skipped} / 失败 {lastSummary.failed}
          </div>
        )}
      </Card>

      <KeywordForm open={formOpen} initial={editing} onCancel={() => setFormOpen(false)} onSubmit={handleSubmit} />
    </div>
  );
};

export default KeywordCollect;
