import React, { useState } from 'react';
import { Button, Card, Empty, Modal, Tabs, Tag, message } from 'antd';
import { AppstoreOutlined, CheckOutlined } from '@ant-design/icons';
import {
  cleaningTemplates,
  cmsTemplates,
  sourceTemplates,
  type CleaningTemplate,
  type SourceTemplate,
} from '@/data/templates';
import type { CollectSourceInput } from '@/types/keyword';
import type { CmsConnectionInput } from '@/types/publish';

interface TemplateLibraryProps {
  open: boolean;
  onClose: () => void;
  /** source：采集源模板；cms：CMS 配置模板 */
  mode: 'source' | 'cms';
  /** mode=source 时应用采集源模板 */
  onApplySource?: (values: Partial<CollectSourceInput>) => void;
  /** mode=source 时应用清洗规则（追加到移除列表） */
  onApplyCleaning?: (removeSelectors: string[]) => void;
  /** mode=cms 时应用 CMS 配置模板 */
  onApplyCms?: (values: Partial<CmsConnectionInput>) => void;
}

const categoryColor: Record<string, string> = {
  社区博客: 'blue',
  新闻资讯: 'geekblue',
  'RSS 订阅': 'orange',
};

/** 模板库：内置采集源 / 清洗规则 / CMS 配置模板，一键应用到表单 */
const TemplateLibrary: React.FC<TemplateLibraryProps> = ({
  open,
  onClose,
  mode,
  onApplySource,
  onApplyCleaning,
  onApplyCms,
}) => {
  const [tab, setTab] = useState('source');

  const handleApplySource = (t: SourceTemplate) => {
    onApplySource?.(t.values);
    message.success(`已应用模板「${t.name}」，请补充 URL 等信息`);
    onClose();
  };

  const handleApplyCleaning = (t: CleaningTemplate) => {
    onApplyCleaning?.(t.removeSelectors);
    message.success(`已追加清洗规则「${t.name}」`);
  };

  const handleApplyCms = (key: string) => {
    const t = cmsTemplates.find((c) => c.key === key);
    if (t) {
      onApplyCms?.(t.values);
      message.success(`已应用配置模板「${t.name}」，请填写站点地址与账号`);
      onClose();
    }
  };

  if (mode === 'cms') {
    return (
      <Modal title="📦 CMS 配置模板库" open={open} onCancel={onClose} footer={null} width={560}>
        <div className="space-y-2 py-2">
          {cmsTemplates.map((t) => (
            <Card
              key={t.key}
              size="small"
              hoverable
              className="cursor-pointer"
              onClick={() => handleApplyCms(t.key)}
            >
              <div className="flex items-center justify-between">
                <span className="font-medium">{t.name}</span>
                <Button type="link" size="small" icon={<CheckOutlined />}>
                  应用
                </Button>
              </div>
              <div className="text-xs text-slate-500">{t.description}</div>
              {t.values.api_path && (
                <div className="mt-1 text-xs text-slate-400">API：{t.values.api_path}</div>
              )}
            </Card>
          ))}
        </div>
      </Modal>
    );
  }

  return (
    <Modal
      title="📦 模板库"
      open={open}
      onCancel={onClose}
      footer={null}
      width={640}
    >
      <Tabs
        activeKey={tab}
        onChange={setTab}
        items={[
          {
            key: 'source',
            label: '采集源模板',
            children: (
              <div className="max-h-[420px] space-y-2 overflow-auto py-1">
                {sourceTemplates.map((t) => (
                  <Card key={t.key} size="small" hoverable onClick={() => handleApplySource(t)}>
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <AppstoreOutlined className="text-maple-500" />
                        <span className="font-medium">{t.name}</span>
                        <Tag color={categoryColor[t.category]}>{t.category}</Tag>
                      </div>
                      <Button type="link" size="small" icon={<CheckOutlined />}>
                        应用
                      </Button>
                    </div>
                    <div className="mt-1 text-xs text-slate-500">{t.description}</div>
                    {t.values.selector_content && (
                      <div className="mt-1 truncate font-mono text-xs text-slate-400">
                        正文：{t.values.selector_content}
                      </div>
                    )}
                  </Card>
                ))}
              </div>
            ),
          },
          {
            key: 'cleaning',
            label: '清洗规则模板',
            children: (
              <div className="max-h-[420px] space-y-2 overflow-auto py-1">
                {cleaningTemplates.map((t) => (
                  <Card key={t.key} size="small" hoverable onClick={() => handleApplyCleaning(t)}>
                    <div className="flex items-center justify-between">
                      <span className="font-medium">{t.name}</span>
                      <Button type="link" size="small">
                        追加规则
                      </Button>
                    </div>
                    <div className="mt-1 text-xs text-slate-500">{t.description}</div>
                    <div className="mt-1 text-xs text-slate-400">{t.removeSelectors.length} 条移除规则</div>
                  </Card>
                ))}
              </div>
            ),
          },
        ]}
      />
    </Modal>
  );
};

export default TemplateLibrary;
