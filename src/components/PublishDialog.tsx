import React from 'react';
import { Modal, Select, Input, Radio, Space, Typography } from 'antd';
import type { PublishTarget } from '@/types/publish';
import { useSettingsStore } from '@/stores/settingsStore';

interface PublishDialogProps {
  open: boolean;
  articleIds: number[];
  connections: { id: number; name: string; cms_type: string }[];
  onCancel: () => void;
  onSubmit: (target: PublishTarget) => Promise<void>;
}

/** 发布弹窗：选择发布方式（Hexo 导出 / CMS 自动发布） */
const PublishDialog: React.FC<PublishDialogProps> = ({ open, articleIds, connections, onCancel, onSubmit }) => {
  const { settings } = useSettingsStore();
  const [publishType, setPublishType] = React.useState<'hexo' | 'cms'>('hexo');
  const [cmsId, setCmsId] = React.useState<number | undefined>(connections[0]?.id);
  const [outputDir, setOutputDir] = React.useState('');
  const [submitting, setSubmitting] = React.useState(false);

  React.useEffect(() => {
    if (open) {
      setPublishType('hexo');
      setCmsId(connections[0]?.id);
      setOutputDir(settings['hexo_output_dir'] || '');
    }
  }, [open, connections, settings]);

  const handleOk = async () => {
    setSubmitting(true);
    try {
      const target: PublishTarget =
        publishType === 'hexo'
          ? { publish_type: 'hexo', output_dir: outputDir || undefined }
          : { publish_type: connections.find((c) => c.id === cmsId)?.cms_type as PublishTarget['publish_type'], cms_id: cmsId };
      await onSubmit(target);
      onCancel();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      title={`发布 ${articleIds.length} 篇文章`}
      open={open}
      onCancel={onCancel}
      onOk={handleOk}
      okText="开始发布"
      confirmLoading={submitting}
      okButtonProps={{ disabled: publishType === 'cms' && !cmsId }}
    >
      <Space direction="vertical" className="w-full" size="middle">
        <Radio.Group value={publishType} onChange={(e) => setPublishType(e.target.value)}>
          <Radio.Button value="hexo">Hexo 导出</Radio.Button>
          <Radio.Button value="cms">CMS 自动发布</Radio.Button>
        </Radio.Group>

        {publishType === 'hexo' ? (
          <>
            <Typography.Text type="secondary">
              将生成带 Front-matter 的 Markdown 文件，导出到 Hexo 博客的 source/_posts 目录。
            </Typography.Text>
            <Input
              placeholder="导出目录，例如 C:/blog/source/_posts（留空使用设置中的默认目录）"
              value={outputDir}
              onChange={(e) => setOutputDir(e.target.value)}
            />
          </>
        ) : (
          <>
            <Typography.Text type="secondary">选择目标 CMS 连接，通过 XML-RPC 自动发布。</Typography.Text>
            <Select
              className="w-full"
              placeholder="选择 CMS 连接"
              value={cmsId}
              onChange={setCmsId}
              options={connections.map((c) => ({
                value: c.id,
                label: `${c.name}（${c.cms_type}）`,
              }))}
              notFoundContent="尚未添加 CMS 连接，请先在发布管理页面添加"
            />
          </>
        )}
      </Space>
    </Modal>
  );
};

export default PublishDialog;
