import React, { useEffect } from 'react';
import { Card, Row, Col, Typography, List, Button, Tag, Space, Modal } from 'antd';
import {
  BookOutlined,
  SettingOutlined,
  QuestionCircleOutlined,
  RocketOutlined,
  FileTextOutlined,
  HighlightOutlined,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import SearchBox from '@/components/SearchBox';
import MarkdownRenderer from '@/components/MarkdownRenderer';
import { useHelpStore } from '@/stores/helpStore';
import { openExternalLink } from '@/services/helpService';

/** 帮助中心首页 */
const HelpCenter: React.FC = () => {
  const navigate = useNavigate();
  const { fetchQuickStart, fetchShortcuts, quickStart } = useHelpStore();
  const [changelogOpen, setChangelogOpen] = React.useState(false);
  const [shortcutOpen, setShortcutOpen] = React.useState(false);

  useEffect(() => {
    void fetchQuickStart();
    void fetchShortcuts();
  }, [fetchQuickStart, fetchShortcuts]);

  const { manualIndex } = useHelpStore();

  const cards = [
    { icon: <BookOutlined className="text-2xl text-blue-500" />, title: '用户手册', desc: '完整的使用指南', onClick: () => navigate('/help/manual') },
    { icon: <SettingOutlined className="text-2xl text-green-500" />, title: '配置示例', desc: '各平台配置模板', onClick: () => navigate('/help/examples') },
    { icon: <QuestionCircleOutlined className="text-2xl text-orange-500" />, title: '常见问题', desc: '按分类解答疑问', onClick: () => navigate('/help/faq') },
    { icon: <RocketOutlined className="text-2xl text-purple-500" />, title: '快速开始', desc: '3 步完成配置', onClick: () => navigate('/help/manual/quickstart') },
    { icon: <FileTextOutlined className="text-2xl text-cyan-500" />, title: '更新日志', desc: '版本更新记录', onClick: () => setChangelogOpen(true) },
    { icon: <HighlightOutlined className="text-2xl text-rose-500" />, title: '快捷键', desc: '快捷键速查表', onClick: () => setShortcutOpen(true) },
  ];

  return (
    <div className="space-y-4">
      <Card size="small" className="bg-gradient-to-r from-maple-50 to-white">
        <div className="py-4">
          <Typography.Title level={4} className="!mb-1">
            ❓ 帮助中心
          </Typography.Title>
          <Typography.Text type="secondary">搜索手册、配置示例与常见问题</Typography.Text>
          <div className="mt-3 max-w-xl">
            <SearchBox onSearch={() => navigate('/help/faq')} showResults />
          </div>
        </div>
      </Card>

      <Row gutter={[16, 16]}>
        {cards.map((c) => (
          <Col span={8} key={c.title}>
            <Card hoverable onClick={c.onClick} className="text-center">
              <div className="mb-2 flex justify-center">{c.icon}</div>
              <div className="font-semibold text-slate-800">{c.title}</div>
              <div className="mt-1 text-xs text-slate-500">{c.desc}</div>
            </Card>
          </Col>
        ))}
      </Row>

      {quickStart && (
        <Card title="🚀 快速开始（3 步）" size="small">
          <Row gutter={16}>
            {quickStart.steps.map((s) => (
              <Col span={8} key={s.step}>
                <Card size="small" className="h-full">
                  <div className="mb-1 flex items-center gap-2">
                    <span className="flex h-6 w-6 items-center justify-center rounded-full bg-maple-500 text-xs font-bold text-white">
                      {s.step}
                    </span>
                    <span className="font-medium">{s.title}</span>
                  </div>
                  <p className="text-sm text-slate-600">{s.description}</p>
                </Card>
              </Col>
            ))}
          </Row>
        </Card>
      )}

      <Card title="热门问题" size="small" extra={<Button type="link" size="small" onClick={() => navigate('/help/faq')}>查看全部</Button>}>
        <List
          size="small"
          dataSource={[
            '如何配置 WordPress 自动发布？',
            '伪原创功能怎么用？',
            '采集时遇到 403 错误怎么办？',
            '如何设置定时采集任务？',
          ]}
          renderItem={(q) => (
            <List.Item className="cursor-pointer hover:text-maple-600" onClick={() => navigate('/help/faq')}>
              <span>❓ {q}</span>
            </List.Item>
          )}
        />
      </Card>

      <Card title="反馈与建议" size="small">
        <Space>
          <Button
            onClick={() => void openExternalLink('https://github.com/xfm0797/x-collector/issues')}
          >
            提交 GitHub Issue
          </Button>
          <Button onClick={() => void openExternalLink('https://github.com/xfm0797/x-collector')}>
            项目主页
          </Button>
        </Space>
      </Card>

      <Modal
        title="📋 更新日志"
        open={changelogOpen}
        onCancel={() => setChangelogOpen(false)}
        footer={null}
        width={640}
      >
        <ChangelogContent />
      </Modal>

      <Modal title="⌨ 快捷键" open={shortcutOpen} onCancel={() => setShortcutOpen(false)} footer={null}>
        <ShortcutTable />
      </Modal>
    </div>
  );
};

/** 更新日志内容 */
const ChangelogContent: React.FC = () => {
  const [changelog, setChangelog] = React.useState('# 加载中...');
  React.useEffect(() => {
    void import('@/services/helpService')
      .then((m) => m.getChangelog())
      .then(setChangelog)
      .catch(() => setChangelog('# 暂无更新日志'));
  }, []);
  return <MarkdownRenderer content={changelog} />;
};

/** 快捷键表 */
const ShortcutTable: React.FC = () => {
  const { shortcuts } = useHelpStore();
  const categories = [...new Set(shortcuts.map((s) => s.category))];
  return (
    <div className="space-y-3">
      {categories.map((cat) => (
        <div key={cat}>
          <Tag color="orange">{cat}</Tag>
          <List
            size="small"
            dataSource={shortcuts.filter((s) => s.category === cat)}
            renderItem={(s) => (
              <List.Item>
                <Tag color="blue">{s.key}</Tag>
                <span className="text-sm">{s.description}</span>
              </List.Item>
            )}
          />
        </div>
      ))}
      {shortcuts.length === 0 && <span className="text-sm text-slate-400">加载中...</span>}
    </div>
  );
};

export default HelpCenter;
