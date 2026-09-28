import React from 'react';
import { Form, Input, Select, Modal, Button, Space, message } from 'antd';
import { testCmsConnection } from '@/services/publishService';
import type { CmsConnectionInput, CmsConnection } from '@/types/publish';

interface CmsConfigFormProps {
  open: boolean;
  initial?: CmsConnection | null;
  onCancel: () => void;
  onSubmit: (input: CmsConnectionInput) => Promise<CmsConnection>;
}

const defaultApiPath: Record<string, string> = {
  wordpress: '/xmlrpc.php',
  typecho: '/action/xmlrpc',
  zblog: '/zb_system/xml-rpc/index.php',
  custom: '',
};

/** CMS 连接配置表单 */
const CmsConfigForm: React.FC<CmsConfigFormProps> = ({ open, initial, onCancel, onSubmit }) => {
  const [form] = Form.useForm();
  const [submitting, setSubmitting] = React.useState(false);
  const [testing, setTesting] = React.useState(false);

  React.useEffect(() => {
    if (open) {
      form.resetFields();
      form.setFieldsValue({
        name: '',
        cms_type: 'wordpress',
        site_url: '',
        api_path: '/xmlrpc.php',
        username: '',
        password: '',
        default_category: '',
        default_status: 'publish',
        enabled: true,
        ...(initial
          ? {
              name: initial.name,
              cms_type: initial.cms_type,
              site_url: initial.site_url,
              api_path: initial.api_path,
              username: initial.username,
              default_category: initial.default_category,
              default_status: initial.default_status,
              enabled: initial.enabled === 1,
            }
          : {}),
      });
    }
  }, [open, initial, form]);

  const handleOk = async () => {
    const values = await form.validateFields();
    setSubmitting(true);
    try {
      await onSubmit({
        ...values,
        id: initial?.id,
        tag_mapping: undefined,
        enabled: values.enabled ? 1 : 0,
        password: values.password || undefined,
      });
      onCancel();
    } finally {
      setSubmitting(false);
    }
  };

  const handleTest = async () => {
    const values = await form.validateFields(['site_url', 'api_path', 'username', 'password']);
    setTesting(true);
    try {
      // 先保存再测试需要 id，这里直接提示用户保存后测试
      const input: CmsConnectionInput = {
        ...values,
        name: form.getFieldValue('name') || '临时连接',
        cms_type: form.getFieldValue('cms_type'),
        default_status: 'publish',
        password: values.password,
      };
      const saved = await onSubmit(input);
      const result = await testCmsConnection(saved.id);
      if (result.success) {
        message.success(result.message);
      } else {
        message.error(result.message);
      }
    } catch (e) {
      message.error(String(e));
    } finally {
      setTesting(false);
    }
  };

  return (
    <Modal
      title={initial ? '编辑 CMS 连接' : '添加 CMS 连接'}
      open={open}
      onCancel={onCancel}
      destroyOnClose
      footer={
        <Space>
          <Button onClick={onCancel}>取消</Button>
          <Button loading={testing} onClick={handleTest}>
            保存并测试
          </Button>
          <Button type="primary" loading={submitting} onClick={handleOk}>
            保存
          </Button>
        </Space>
      }
    >
      <Form form={form} layout="vertical">
        <Form.Item name="name" label="连接名称" rules={[{ required: true, message: '请输入连接名称' }]}>
          <Input placeholder="例如：我的 WordPress 博客" />
        </Form.Item>
        <Form.Item name="cms_type" label="CMS 类型">
          <Select
            options={[
              { value: 'wordpress', label: 'WordPress' },
              { value: 'typecho', label: 'Typecho' },
              { value: 'zblog', label: 'Z-Blog' },
              { value: 'custom', label: '自定义（XML-RPC）' },
            ]}
            onChange={(v) => form.setFieldValue('api_path', defaultApiPath[v] || '')}
          />
        </Form.Item>
        <Form.Item name="site_url" label="站点地址" rules={[{ required: true, message: '请输入站点地址' }]}>
          <Input placeholder="https://yourblog.com" />
        </Form.Item>
        <Form.Item name="api_path" label="API 路径">
          <Input placeholder="/xmlrpc.php" />
        </Form.Item>
        <div className="grid grid-cols-2 gap-3">
          <Form.Item name="username" label="用户名" rules={[{ required: true, message: '请输入用户名' }]}>
            <Input placeholder="admin" />
          </Form.Item>
          <Form.Item
            name="password"
            label={initial ? '密码（留空保持不变）' : '密码 / 应用密码'}
            rules={initial ? [] : [{ required: true, message: '请输入密码' }]}
          >
            <Input.Password placeholder="WordPress 建议使用应用密码" />
          </Form.Item>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Form.Item name="default_category" label="默认分类">
            <Input placeholder="技术" />
          </Form.Item>
          <Form.Item name="default_status" label="默认发布状态">
            <Select
              options={[
                { value: 'publish', label: '直接发布' },
                { value: 'draft', label: '保存为草稿' },
              ]}
            />
          </Form.Item>
        </div>
      </Form>
    </Modal>
  );
};

export default CmsConfigForm;
