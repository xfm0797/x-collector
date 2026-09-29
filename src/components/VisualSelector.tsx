import React, { useMemo, useState } from 'react';
import { Button, Empty, Input, Modal, Space, Spin, Tag, Typography, message } from 'antd';
import { AimOutlined, PlayCircleOutlined, SearchOutlined } from '@ant-design/icons';
import type { DataNode } from 'antd/es/tree';
import Tree from 'antd/es/tree';
import {
  inspectPage,
  testSelector,
  type DomNode,
  type PageInspect,
  type SelectorTestResult,
} from '@/services/collectService';

interface VisualSelectorProps {
  open: boolean;
  onClose: () => void;
  /** 初始 URL（通常来自表单） */
  initialUrl?: string;
  /** 选定选择器后回调 */
  onPick: (selector: string) => void;
}

/** DOM 树节点 → antd Tree 数据 */
function toTreeData(node: DomNode): DataNode {
  const label = (
    <span className="text-[13px]">
      <span className="font-medium text-rose-600">{node.tag}</span>
      {node.id && <span className="text-amber-600">#{node.id}</span>}
      {node.classes.length > 0 && (
        <span className="text-sky-600">.{node.classes.slice(0, 2).join('.')}</span>
      )}
      {node.text_preview && (
        <span className="ml-1 text-slate-400"> {node.text_preview.slice(0, 24)}</span>
      )}
      {node.text_length > 500 && (
        <span className="ml-1 rounded bg-emerald-50 px-1 text-[11px] text-emerald-600">
          {node.text_length}字
        </span>
      )}
    </span>
  );
  return {
    key: node.path,
    title: label,
    children: node.children.length > 0 ? node.children.map(toTreeData) : undefined,
  };
}

/** 收集路径 → 节点映射，用于点击时反查 */
function flatten(node: DomNode, map: Map<string, DomNode>): void {
  map.set(node.path, node);
  node.children.forEach((c) => flatten(c, map));
}

/** 可视化选择器：加载页面 DOM 树 → 点击节点生成 CSS 选择器 → 实时测试 → 回填 */
const VisualSelector: React.FC<VisualSelectorProps> = ({ open, onClose, initialUrl, onPick }) => {
  const [url, setUrl] = useState('');
  const [loading, setLoading] = useState(false);
  const [page, setPage] = useState<PageInspect | null>(null);
  const [selected, setSelected] = useState<DomNode | null>(null);
  const [selector, setSelector] = useState('');
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<SelectorTestResult | null>(null);

  React.useEffect(() => {
    if (open) {
      setUrl(initialUrl || '');
      setPage(null);
      setSelected(null);
      setResult(null);
      setSelector('');
    }
  }, [open, initialUrl]);

  const nodeMap = useMemo(() => {
    const map = new Map<string, DomNode>();
    if (page) flatten(page.dom, map);
    return map;
  }, [page]);

  const treeData = useMemo(() => (page ? [toTreeData(page.dom)] : []), [page]);

  const handleLoad = async () => {
    if (!url.trim().startsWith('http')) {
      message.warning('请输入以 http/https 开头的 URL');
      return;
    }
    setLoading(true);
    setSelected(null);
    setResult(null);
    try {
      const data = await inspectPage(url.trim());
      setPage(data);
      if (data.dom.children.length === 0) {
        message.warning('页面未解析出 DOM 结构，可能为纯脚本渲染页面');
      }
    } catch (e) {
      message.error(`页面加载失败：${String(e)}`);
      setPage(null);
    } finally {
      setLoading(false);
    }
  };

  const handleSelect = (keys: React.Key[]) => {
    const key = String(keys[0] || '');
    const node = nodeMap.get(key);
    if (node) {
      setSelected(node);
      // 默认使用节点自身选择器片段（简洁），可手动改为完整路径
      setSelector(node.css);
      setResult(null);
    }
  };

  const handleTest = async () => {
    if (!selector.trim()) {
      message.warning('请先在 DOM 树中选择节点或输入选择器');
      return;
    }
    setTesting(true);
    try {
      const res = await testSelector(url.trim(), selector.trim());
      setResult(res);
      if (res.matched === 0) {
        message.warning('该选择器未匹配到任何元素，请尝试使用完整路径');
      }
    } catch (e) {
      message.error(`测试失败：${String(e)}`);
      setResult(null);
    } finally {
      setTesting(false);
    }
  };

  const handlePick = () => {
    if (!selector.trim()) return;
    onPick(selector.trim());
    onClose();
  };

  return (
    <Modal
      title="🎯 可视化选择器"
      open={open}
      onCancel={onClose}
      width={960}
      footer={
        <Space>
          <Button onClick={onClose}>取消</Button>
          <Button type="primary" icon={<AimOutlined />} disabled={!selector.trim()} onClick={handlePick}>
            使用此选择器
          </Button>
        </Space>
      }
    >
      <Space.Compact style={{ width: '100%' }} className="mb-3">
        <Input
          prefix={<SearchOutlined />}
          placeholder="输入示例文章页 URL（如 https://example.com/post/123）"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onPressEnter={handleLoad}
        />
        <Button type="primary" loading={loading} onClick={handleLoad}>
          加载页面
        </Button>
      </Space.Compact>

      {page && (
        <div className="mb-2 truncate text-xs text-slate-500">
          页面标题：<span className="text-slate-700">{page.title}</span>
        </div>
      )}

      <div className="grid grid-cols-[1fr_340px] gap-3">
        <div className="max-h-[420px] overflow-auto rounded border border-slate-200 p-2">
          {loading && (
            <div className="flex h-40 items-center justify-center">
              <Spin tip="正在抓取页面…" />
            </div>
          )}
          {!loading && !page && (
            <Empty description="加载页面后在此显示 DOM 结构，点击节点即可生成选择器" image={Empty.PRESENTED_IMAGE_SIMPLE} />
          )}
          {!loading && page && (
            <Tree
              treeData={treeData}
              defaultExpandedKeys={page.dom.children.length > 0 ? [page.dom.children[0]!.path] : []}
              showLine={{ showLeafIcon: false }}
              onSelect={handleSelect}
              selectedKeys={selected ? [selected.path] : []}
              blockNode
            />
          )}
        </div>

        <div className="max-h-[420px] space-y-3 overflow-auto">
          {selected && (
            <div className="rounded border border-slate-200 p-2 text-[13px]">
              <div className="mb-1 flex flex-wrap items-center gap-1">
                <Tag color="red">{selected.tag}</Tag>
                {selected.id && <Tag color="orange">#{selected.id}</Tag>}
                {selected.classes.slice(0, 3).map((c) => (
                  <Tag key={c} color="blue">
                    .{c}
                  </Tag>
                ))}
              </div>
              {selected.text_preview && (
                <div className="text-slate-500">文本：{selected.text_preview}</div>
              )}
              <div className="text-slate-400">内文本总量：{selected.text_length} 字符</div>
            </div>
          )}

          <div>
            <Typography.Text type="secondary" className="text-xs">
              CSS 选择器（可编辑）
            </Typography.Text>
            <Input.TextArea
              rows={2}
              value={selector}
              onChange={(e) => setSelector(e.target.value)}
              placeholder="选择节点后自动生成，也可手动编辑"
              className="mt-1"
            />
            {selected && selected.css !== selected.path && (
              <div className="mt-1 flex items-center gap-1 text-xs">
                <span className="text-slate-400">完整路径：</span>
                <Button
                  type="link"
                  size="small"
                  className="!h-auto !p-0 text-xs"
                  onClick={() => setSelector(selected.path)}
                >
                  使用完整路径
                </Button>
              </div>
            )}
            <Button
              className="mt-2"
              icon={<PlayCircleOutlined />}
              loading={testing}
              disabled={!selector.trim()}
              onClick={handleTest}
              block
            >
              测试选择器
            </Button>
          </div>

          {result && (
            <div className="rounded border border-slate-200 p-2 text-[13px]">
              <div className="mb-1">
                匹配 <span className="font-semibold text-emerald-600">{result.matched}</span> 个元素
                {result.matched === 0 && <span className="text-rose-500">（建议改用完整路径）</span>}
              </div>
              <div className="max-h-40 space-y-1 overflow-auto">
                {result.matches.map((m) => (
                  <div key={m.index} className="rounded bg-slate-50 p-1.5">
                    <div className="text-[11px] text-slate-400">
                      #{m.index} · {m.html_length} 字节 HTML
                    </div>
                    <div className="line-clamp-2 text-slate-600">{m.text_preview || '（无文本）'}</div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
};

export default VisualSelector;
