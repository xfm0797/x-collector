import React from 'react';
import { Form, Input, Select, InputNumber, Switch, Modal } from 'antd';
import type { KeywordTaskInput } from '@/types/keyword';

interface KeywordFormProps {
  open: boolean;
  initial?: Partial<KeywordTaskInput>;
  onCancel: () => void;
  onSubmit: (input: KeywordTaskInput) => Promise<void>;
}

/** 关键词任务表单（新增/编辑弹窗） */
const KeywordForm: React.FC<KeywordFormProps> = ({ open, initial, onCancel, onSubmit }) => {
  const [form] = Form.useForm();
  const [submitting, setSubmitting] = React.useState(false);

  React.useEffect(() => {
    if (open) {
      form.resetFields();
      form.setFieldsValue({
        keyword: '',
        group_name: '默认',
        search_engine: 'baidu',
        match_mode: 'title',
        max_pages: 3,
        interval_minutes: 0,
        enabled: true,
        site_limit: '',
        ...initial,
      });
    }
  }, [open, initial, form]);

  const handleOk = async () => {
    const values = await form.validateFields();
    setSubmitting(true);
    try {
      await onSubmit({
        ...values,
        site_limit: values.site_limit || undefined,
        enabled: values.enabled ? 1 : 0,
      });
      onCancel();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      title={initial?.keyword ? '编辑关键词任务' : '添加关键词任务'}
      open={open}
      onOk={handleOk}
      onCancel={onCancel}
      confirmLoading={submitting}
      destroyOnClose
    >
      <Form form={form} layout="vertical" initialValues={{ max_pages: 3 }}>
        <Form.Item name="keyword" label="关键词" rules={[{ required: true, message: '请输入关键词' }]}>
          <Input placeholder="例如：Rust 教程" />
        </Form.Item>
        <div className="grid grid-cols-2 gap-3">
          <Form.Item name="group_name" label="分组">
            <Input placeholder="默认" />
          </Form.Item>
          <Form.Item name="search_engine" label="搜索引擎">
            <Select
              options={[
                { value: 'baidu', label: '百度' },
                { value: 'google', label: 'Google' },
                { value: 'bing', label: 'Bing' },
                { value: 'sogou', label: '搜狗' },
                { value: 'custom', label: '自定义' },
              ]}
            />
          </Form.Item>
        </div>
        <Form.Item name="site_limit" label="站点限定（可选）">
          <Input placeholder="例如：csdn.net，将附加 site: 查询" />
        </Form.Item>
        <div className="grid grid-cols-2 gap-3">
          <Form.Item name="match_mode" label="匹配模式">
            <Select
              options={[
                { value: 'title', label: '标题匹配' },
                { value: 'content', label: '全文匹配' },
                { value: 'title_or_content', label: '标题或全文' },
                { value: 'exact', label: '精确匹配' },
              ]}
            />
          </Form.Item>
          <Form.Item name="max_pages" label="采集深度（页）">
            <InputNumber min={1} max={10} className="w-full" />
          </Form.Item>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Form.Item
            name="interval_minutes"
            label="轮询间隔（分钟）"
            tooltip="0 表示不自动轮询"
          >
            <InputNumber min={0} max={1440} className="w-full" />
          </Form.Item>
          <Form.Item name="enabled" label="启用" valuePropName="checked">
            <Switch />
          </Form.Item>
        </div>
      </Form>
    </Modal>
  );
};

export default KeywordForm;
