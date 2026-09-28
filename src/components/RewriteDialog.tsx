import React from 'react';
import { Modal, Form, Select, Slider, Switch, Input, Typography } from 'antd';
import type { RewriteOptions } from '@/types/rewrite';

interface RewriteDialogProps {
  open: boolean;
  options: RewriteOptions;
  onCancel: () => void;
  onSubmit: (options: RewriteOptions) => Promise<void>;
}

/** 伪原创设置弹窗 */
const RewriteDialog: React.FC<RewriteDialogProps> = ({ open, options, onCancel, onSubmit }) => {
  const [form] = Form.useForm();
  const [submitting, setSubmitting] = React.useState(false);

  React.useEffect(() => {
    if (open && options) {
      form.resetFields();
      form.setFieldsValue({
        ...options,
        keywords: (options.keywords || []).join('，'),
      });
    }
  }, [open, options, form]);

  const handleOk = async () => {
    const values = await form.validateFields();
    setSubmitting(true);
    try {
      await onSubmit({
        enabled: values.enabled,
        intensity: values.intensity,
        synonym_ratio: values.synonym_ratio,
        sentence_ratio: values.sentence_ratio,
        paragraph_shuffle: values.paragraph_shuffle,
        rewrite_ends: values.rewrite_ends,
        keywords: String(values.keywords || '')
          .split(/[,，\s]+/)
          .filter(Boolean),
      });
      onCancel();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal title="伪原创设置" open={open} onCancel={onCancel} onOk={handleOk} confirmLoading={submitting} destroyOnClose>
      <Form form={form} layout="vertical">
        <Form.Item name="enabled" label="启用伪原创" valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item name="intensity" label="改写强度">
          <Select
            options={[
              { value: 'light', label: '轻度' },
              { value: 'medium', label: '中等' },
              { value: 'heavy', label: '深度' },
            ]}
          />
        </Form.Item>
        <Form.Item name="synonym_ratio" label="同义词替换比例">
          <Slider min={10} max={50} marks={{ 10: '10%', 30: '30%', 50: '50%' }} />
        </Form.Item>
        <Form.Item name="sentence_ratio" label="句子改写比例">
          <Slider min={10} max={50} marks={{ 10: '10%', 20: '20%', 50: '50%' }} />
        </Form.Item>
        <Form.Item name="paragraph_shuffle" label="段落重排（保留首尾段）" valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item name="rewrite_ends" label="首尾段重写" valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item name="keywords" label="插入关键词（逗号分隔）">
          <Input placeholder="例如：Rust，系统编程" />
        </Form.Item>
        <Typography.Text type="secondary" className="text-xs">
          首尾段重写将使用内置模板库重新生成开头结尾；代码块与表格不会被改写。
        </Typography.Text>
      </Form>
    </Modal>
  );
};

export default RewriteDialog;
