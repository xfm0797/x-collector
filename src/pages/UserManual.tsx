import React, { useEffect } from 'react';
import { Card, Menu, Button, Space, Spin, Empty, Typography } from 'antd';
import { LeftOutlined, RightOutlined, HomeOutlined } from '@ant-design/icons';
import { useParams, useNavigate } from 'react-router-dom';
import MarkdownRenderer from '@/components/MarkdownRenderer';
import TocSidebar from '@/components/TocSidebar';
import SearchBox from '@/components/SearchBox';
import { useHelpStore } from '@/stores/helpStore';
import type { ManualSection } from '@/types/help';

/** 用户手册页面：左侧目录树 + 右侧内容 + 上下页导航 */
const UserManual: React.FC = () => {
  const { section = 'index' } = useParams();
  const navigate = useNavigate();
  const { manualIndex, manualContent, isLoadingManual, fetchManualIndex, fetchManualSection } = useHelpStore();

  useEffect(() => {
    if (manualIndex.length === 0) {
      void fetchManualIndex();
    }
  }, [manualIndex.length, fetchManualIndex]);

  useEffect(() => {
    void fetchManualSection(section);
  }, [section, fetchManualSection]);

  /** 扁平化目录用于上一页/下一页 */
  const flat = (nodes: ManualSection[]): ManualSection[] =>
    nodes.flatMap((n) => [n, ...flat(n.children || [])]);
  const flatSections = flat(manualIndex).filter((s) => !s.path.includes('#'));
  const currentIndex = flatSections.findIndex((s) => s.path === section);
  const prev = currentIndex > 0 ? flatSections[currentIndex - 1] : null;
  const next = currentIndex >= 0 && currentIndex < flatSections.length - 1 ? flatSections[currentIndex + 1] : null;

  const menuItems = (nodes: ManualSection[]): any[] =>
    nodes.map((n) => ({
      key: n.path,
      label: n.title,
      children: n.children && n.children.length > 0 ? menuItems(n.children) : undefined,
    }));

  return (
    <div className="flex h-full gap-3">
      <Card size="small" className="w-60 shrink-0 overflow-auto" styles={{ body: { padding: 4 } }} style={{ maxHeight: 'calc(100vh - 120px)' }}>
        <div className="flex items-center justify-between px-2 py-1">
          <span className="text-sm font-semibold text-slate-700">📖 用户手册</span>
          <Button type="text" size="small" icon={<HomeOutlined />} onClick={() => navigate('/help')} />
        </div>
        <div className="px-1 pb-2">
          <SearchBox placeholder="搜索本页..." onSearch={(q) => navigate('/help/faq')} showResults={false} />
        </div>
        {manualIndex.length > 0 && (
          <Menu
            mode="inline"
            selectedKeys={[section]}
            defaultOpenKeys={manualIndex.map((m) => m.path)}
            items={menuItems(manualIndex)}
            onClick={({ key }) => navigate(`/help/manual/${key}`)}
            className="!border-r-0"
            style={{ borderInlineEnd: 'none' }}
          />
        )}
      </Card>

      <div className="flex flex-1 gap-3 overflow-hidden">
        <Card size="small" className="flex-1 overflow-auto" style={{ maxHeight: 'calc(100vh - 120px)' }} styles={{ body: { padding: '8px 20px' } }}>
          {isLoadingManual ? (
            <div className="flex justify-center py-20">
              <Spin />
            </div>
          ) : manualContent ? (
            <>
              <MarkdownRenderer content={manualContent.content} />
              <div className="mt-8 flex items-center justify-between border-t border-slate-200 pt-4">
                <Space>
                  <Button
                    disabled={!prev}
                    icon={<LeftOutlined />}
                    onClick={() => prev && navigate(`/help/manual/${prev.path}`)}
                  >
                    {prev ? prev.title : '已是第一页'}
                  </Button>
                </Space>
                <Space>
                  <Button
                    disabled={!next}
                    onClick={() => next && navigate(`/help/manual/${next.path}`)}
                  >
                    {next ? next.title : '已是最后一页'}
                    <RightOutlined />
                  </Button>
                </Space>
              </div>
            </>
          ) : (
            <Empty description="未找到该章节" />
          )}
        </Card>
        {manualContent && (
          <div className="overflow-auto" style={{ maxHeight: 'calc(100vh - 120px)' }}>
            <TocSidebar content={manualContent.content} />
          </div>
        )}
      </div>
    </div>
  );
};

export default UserManual;
