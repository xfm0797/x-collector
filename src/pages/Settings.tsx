import React, { useEffect, useState } from 'react';
import { Card, Tabs, Form, Input, Select, InputNumber, Switch, Slider, Button, Space, message, Typography, Modal, Alert } from 'antd';
import { SaveOutlined, ImportOutlined, DownloadOutlined } from '@ant-design/icons';
import { useSettingsStore } from '@/stores/settingsStore';
import { useRewrite } from '@/hooks/useRewrite';
import { importCustomDict, resetCustomDict, getDictStats } from '@/services/rewriteService';
import type { RewriteOptions } from '@/types/rewrite';

/** 设置页面：通用/采集/发布/伪原创/代理 */
const Settings: React.FC = () => {
  const { settings, saving, fetchSettings, saveSettings } = useSettingsStore();
  const { options, fetchOptions, saveOptions } = useRewrite();
  const [dictOpen, setDictOpen] = useState(false);
  const [dictContent, setDictContent] = useState('');
  const [dictStats, setDictStats] = useState<{ builtin_groups: number; custom_groups: number } | null>(null);

  useEffect(() => {
    void fetchSettings();
    void fetchOptions();
    void getDictStats().then(setDictStats).catch(() => undefined);
  }, [fetchSettings, fetchOptions]);

  const get = (key: string, fallback = '') => settings[key] ?? fallback;
  const num = (key: string, fallback: number) => Number(settings[key] ?? fallback) || fallback;

  const handleSave = async (patch: Record<string, string>) => {
    await saveSettings(patch);
    message.success('设置已保存');
  };

  /** 导出全部设置为 JSON */
  const handleExport = () => {
    const blob = new Blob([JSON.stringify(settings, null, 2)], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'x-collector-settings.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImport = async (file: File) => {
    try {
      const text = await file.text();
      const data = JSON.parse(text);
      await saveSettings(data);
      message.success('设置已导入');
    } catch (e) {
      message.error(`导入失败：${String(e)}`);
    }
    return false;
  };

  const handleSaveRewrite = async (o: Partial<RewriteOptions>) => {
    if (!options) return;
    await saveOptions({ ...options, ...o });
    message.success('伪原创设置已保存');
  };

  return (
    <div className="space-y-3">
      <Card size="small">
        <div className="flex items-center justify-between">
          <span className="text-base font-semibold">⚙ 系统设置</span>
          <Space>
            <Button icon={<DownloadOutlined />} onClick={handleExport}>
              导出设置
            </Button>
            <label className="ant-btn ant-btn-default cursor-pointer">
              <ImportOutlined /> 导入设置
              <input
                type="file"
                accept=".json"
                className="hidden"
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  if (f) void handleImport(f);
                }}
              />
            </label>
          </Space>
        </div>
      </Card>

      <Card size="small">
        <Tabs
          items={[
            {
              key: 'general',
              label: '通用',
              children: (
                <GeneralTab settings={settings} get={get} onSave={handleSave} saving={saving} />
              ),
            },
            {
              key: 'collect',
              label: '采集',
              children: <CollectTab settings={settings} num={num} get={get} onSave={handleSave} saving={saving} />,
            },
            {
              key: 'publish',
              label: '发布',
              children: <PublishTab settings={settings} get={get} onSave={handleSave} saving={saving} />,
            },
            {
              key: 'rewrite',
              label: '伪原创',
              children: options ? (
                <RewriteTab
                  options={options}
                  dictStats={dictStats}
                  onSave={handleSaveRewrite}
                  onOpenDict={() => setDictOpen(true)}
                  onResetDict={async () => {
                    await resetCustomDict();
                    setDictStats(await getDictStats());
                    message.success('已重置为默认词库');
                  }}
                />
              ) : (
                <span className="text-slate-400">加载中...</span>
              ),
            },
            {
              key: 'proxy',
              label: '代理',
              children: <ProxyTab settings={settings} get={get} onSave={handleSave} saving={saving} />,
            },
          ]}
        />
      </Card>

      <Modal
        title="导入自定义同义词词库"
        open={dictOpen}
        onCancel={() => setDictOpen(false)}
        okText="导入"
        onOk={async () => {
          try {
            const result = await importCustomDict(dictContent);
            if (result.success) {
              message.success(result.message);
              setDictOpen(false);
              setDictStats(await getDictStats());
            } else {
              message.error(result.message);
            }
          } catch (e) {
            message.error(String(e));
          }
        }}
      >
        <Alert
          type="info"
          showIcon
          className="mb-2"
          message='格式：JSON 二维数组，如 [["快速","迅速","敏捷"],["重要","关键","核心"]]'
        />
        <Input.TextArea rows={10} value={dictContent} onChange={(e) => setDictContent(e.target.value)} placeholder='[["词","同义词1","同义词2"]]' />
      </Modal>
    </div>
  );
};

interface TabProps {
  settings: Record<string, string>;
  get: (key: string, fallback?: string) => string;
  num?: (key: string, fallback: number) => number;
  onSave: (patch: Record<string, string>) => Promise<void>;
  saving: boolean;
}

const GeneralTab: React.FC<TabProps> = ({ settings, get, onSave, saving }) => {
  const [lang, setLang] = useState(get('language', 'zh'));
  const [theme, setTheme] = useState(get('theme', 'light'));
  const [cache, setCache] = useState(get('cache_enabled', 'true') === 'true');
  useEffect(() => {
    setLang(settings.language || 'zh');
    setTheme(settings.theme || 'light');
    setCache((settings.cache_enabled ?? 'true') === 'true');
  }, [settings]);
  return (
    <Form layout="vertical" className="max-w-md">
      <Form.Item label="语言">
        <Select value={lang} onChange={setLang} options={[{ value: 'zh', label: '简体中文' }, { value: 'en', label: 'English（开发中）' }]} />
      </Form.Item>
      <Form.Item label="主题">
        <Select value={theme} onChange={setTheme} options={[{ value: 'light', label: '浅色' }, { value: 'dark', label: '深色（开发中）' }]} />
      </Form.Item>
      <Form.Item label="启用缓存（采集内容本地缓存）">
        <Switch checked={cache} onChange={setCache} />
      </Form.Item>
      <Button type="primary" icon={<SaveOutlined />} loading={saving} onClick={() => onSave({ language: lang, theme, cache_enabled: String(cache) })}>
        保存通用设置
      </Button>
    </Form>
  );
};

const CollectTab: React.FC<TabProps> = ({ settings, num, get, onSave, saving }) => {
  const [timeout_, setTimeout_] = useState(num?.('collect_timeout', 30) ?? 30);
  const [concurrency, setConcurrency] = useState(num?.('collect_concurrency', 3) ?? 3);
  const [intervalMin, setIntervalMin] = useState(num?.('collect_interval_min', 2) ?? 2);
  const [intervalMax, setIntervalMax] = useState(num?.('collect_interval_max', 5) ?? 5);
  const [retries, setRetries] = useState(num?.('collect_retries', 3) ?? 3);
  const [ua, setUa] = useState(get('collect_user_agent', ''));
  useEffect(() => {
    setTimeout_(Number(settings.collect_timeout ?? 30) || 30);
    setConcurrency(Number(settings.collect_concurrency ?? 3) || 3);
    setIntervalMin(Number(settings.collect_interval_min ?? 2) || 2);
    setIntervalMax(Number(settings.collect_interval_max ?? 5) || 5);
    setRetries(Number(settings.collect_retries ?? 3) || 3);
    setUa(settings.collect_user_agent ?? '');
  }, [settings]);
  return (
    <Form layout="vertical" className="max-w-md">
      <Form.Item label="请求超时（秒）">
        <InputNumber min={5} max={120} value={timeout_} onChange={(v) => setTimeout_(v || 30)} className="w-full" />
      </Form.Item>
      <Form.Item label="并发数">
        <InputNumber min={1} max={10} value={concurrency} onChange={(v) => setConcurrency(v || 3)} className="w-full" />
      </Form.Item>
      <Form.Item label="请求间隔（秒，随机延迟范围）">
        <Space>
          <InputNumber min={0} max={30} value={intervalMin} onChange={(v) => setIntervalMin(v || 0)} />
          <span>至</span>
          <InputNumber min={0} max={60} value={intervalMax} onChange={(v) => setIntervalMax(v || 5)} />
        </Space>
      </Form.Item>
      <Form.Item label="失败重试次数">
        <InputNumber min={0} max={10} value={retries} onChange={(v) => setRetries(v || 0)} className="w-full" />
      </Form.Item>
      <Form.Item label="自定义 User-Agent（留空使用随机轮换）">
        <Input value={ua} onChange={(e) => setUa(e.target.value)} placeholder="Mozilla/5.0 (Windows NT 10.0; Win64; x64)..." />
      </Form.Item>
      <Button
        type="primary"
        icon={<SaveOutlined />}
        loading={saving}
        onClick={() =>
          onSave({
            collect_timeout: String(timeout_),
            collect_concurrency: String(concurrency),
            collect_interval_min: String(intervalMin),
            collect_interval_max: String(intervalMax),
            collect_retries: String(retries),
            collect_user_agent: ua,
          })
        }
      >
        保存采集设置
      </Button>
    </Form>
  );
};

const PublishTab: React.FC<TabProps> = ({ settings, get, onSave, saving }) => {
  const [publishType, setPublishType] = useState(get('default_publish_type', 'hexo'));
  const [hexoDir, setHexoDir] = useState(get('hexo_output_dir', ''));
  useEffect(() => {
    setPublishType(settings.default_publish_type || 'hexo');
    setHexoDir(settings.hexo_output_dir ?? '');
  }, [settings]);
  return (
    <Form layout="vertical" className="max-w-lg">
      <Form.Item label="默认发布方式">
        <Select
          value={publishType}
          onChange={setPublishType}
          options={[
            { value: 'hexo', label: 'Hexo 导出（文件导出）' },
            { value: 'wordpress', label: 'WordPress 自动发布（XML-RPC）' },
            { value: 'typecho', label: 'Typecho 自动发布（XML-RPC）' },
            { value: 'zblog', label: 'Z-Blog 自动发布（XML-RPC）' },
            { value: 'custom', label: '自定义 API' },
          ]}
        />
      </Form.Item>
      <Form.Item label="Hexo 默认导出目录">
        <Input value={hexoDir} onChange={(e) => setHexoDir(e.target.value)} placeholder="C:/blog/source/_posts" />
      </Form.Item>
      <Button
        type="primary"
        icon={<SaveOutlined />}
        loading={saving}
        onClick={() => onSave({ default_publish_type: publishType, hexo_output_dir: hexoDir })}
      >
        保存发布设置
      </Button>
    </Form>
  );
};

interface RewriteTabProps {
  options: RewriteOptions;
  dictStats: { builtin_groups: number; custom_groups: number } | null;
  onSave: (options: Partial<RewriteOptions>) => Promise<void>;
  onOpenDict: () => void;
  onResetDict: () => Promise<void>;
}

const RewriteTab: React.FC<RewriteTabProps> = ({ options, dictStats, onSave, onOpenDict, onResetDict }) => {
  const [o, setO] = useState(options);
  useEffect(() => setO(options), [options]);
  return (
    <Form layout="vertical" className="max-w-lg">
      <Typography.Title level={5}>✨ 伪原创设置</Typography.Title>
      <Form.Item label="启用伪原创">
        <Switch checked={o.enabled} onChange={(v) => setO({ ...o, enabled: v })} />
      </Form.Item>
      <Form.Item label="默认改写强度">
        <Select
          value={o.intensity}
          onChange={(v) => setO({ ...o, intensity: v })}
          options={[
            { value: 'light', label: '轻度' },
            { value: 'medium', label: '中等' },
            { value: 'heavy', label: '深度' },
          ]}
        />
      </Form.Item>
      <Form.Item label={`同义词替换比例：${o.synonym_ratio}%`}>
        <Slider min={10} max={50} value={o.synonym_ratio} onChange={(v) => setO({ ...o, synonym_ratio: v })} />
      </Form.Item>
      <Form.Item label={`句子改写比例：${o.sentence_ratio}%`}>
        <Slider min={10} max={50} value={o.sentence_ratio} onChange={(v) => setO({ ...o, sentence_ratio: v })} />
      </Form.Item>
      <Form.Item label="段落重排（保留首尾段）">
        <Switch checked={o.paragraph_shuffle} onChange={(v) => setO({ ...o, paragraph_shuffle: v })} />
      </Form.Item>
      <Form.Item label="首尾段重写">
        <Switch checked={o.rewrite_ends} onChange={(v) => setO({ ...o, rewrite_ends: v })} />
      </Form.Item>

      <Typography.Title level={5}>词库管理</Typography.Title>
      <Typography.Text type="secondary" className="block mb-2">
        当前词库：默认词库（{dictStats?.builtin_groups ?? 0} 组）+ 自定义（{dictStats?.custom_groups ?? 0} 组）
      </Typography.Text>
      <Space className="mb-4">
        <Button onClick={onOpenDict}>导入自定义词库</Button>
        <Button onClick={() => void onResetDict()}>重置为默认</Button>
      </Space>

      <Button type="primary" icon={<SaveOutlined />} onClick={() => void onSave(o)}>
        保存伪原创设置
      </Button>
    </Form>
  );
};

const ProxyTab: React.FC<TabProps> = ({ settings, get, onSave, saving }) => {
  const [proxy, setProxy] = useState(get('proxy_url', ''));
  useEffect(() => setProxy(settings.proxy_url ?? ''), [settings]);
  return (
    <Form layout="vertical" className="max-w-lg">
      <Form.Item label="代理服务器（HTTP / SOCKS5）" tooltip="例如 http://127.0.0.1:7890 或 socks5://127.0.0.1:1080">
        <Input value={proxy} onChange={(e) => setProxy(e.target.value)} placeholder="http://127.0.0.1:7890" />
      </Form.Item>
      <Typography.Text type="secondary" className="block mb-3 text-xs">
        采集与发布请求均将通过该代理发送，留空则直连。
      </Typography.Text>
      <Button type="primary" icon={<SaveOutlined />} loading={saving} onClick={() => onSave({ proxy_url: proxy })}>
        保存代理设置
      </Button>
    </Form>
  );
};

export default Settings;
