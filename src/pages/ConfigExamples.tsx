import React, { useEffect, useState } from 'react';
import { Card, Tabs, Alert, Typography, Radio, Space } from 'antd';
import CodeBlock from '@/components/CodeBlock';
import { useHelpStore } from '@/stores/helpStore';
import type { ConfigExample } from '@/types/help';

const platformLabels: Record<string, string> = {
  wordpress: 'WordPress',
  typecho: 'Typecho',
  hexo: 'Hexo',
  zblog: 'Z-Blog',
  keyword: '关键词采集',
  rewrite: '伪原创',
};

/** 配置示例页面 */
const ConfigExamples: React.FC = () => {
  const { examples, fetchExamples } = useHelpStore();
  const [platform, setPlatform] = useState<string>('wordpress');

  useEffect(() => {
    void fetchExamples();
  }, [fetchExamples]);

  useEffect(() => {
    void fetchExamples(platform === 'all' ? undefined : platform);
  }, [platform, fetchExamples]);

  const platforms = [...new Set(examples.map((e) => e.platform))];
  const filtered = examples.filter((e) => e.platform === platform);

  return (
    <div className="space-y-3">
      <Card size="small" className="bg-gradient-to-r from-maple-50 to-white">
        <Typography.Title level={5} className="!mb-0">
          ⚙ 配置示例
        </Typography.Title>
        <Typography.Text type="secondary">各平台配置模板，可一键复制或下载为 JSON 文件</Typography.Text>
      </Card>

      <Card size="small">
        <Radio.Group value={platform} onChange={(e) => setPlatform(e.target.value)} optionType="button" buttonStyle="solid">
          {(platforms.length > 0 ? platforms : ['wordpress', 'typecho', 'hexo', 'zblog', 'keyword', 'rewrite']).map((p) => (
            <Radio.Button key={p} value={p}>
              {platformLabels[p] || p}
            </Radio.Button>
          ))}
        </Radio.Group>

        <div className="mt-4 space-y-4">
          {filtered.length === 0 ? (
            <Typography.Text type="secondary">该平台暂无示例</Typography.Text>
          ) : (
            filtered.map((example: ConfigExample) => (
              <div key={example.id}>
                <div className="mb-1 flex items-center gap-2">
                  <Typography.Text strong>{example.title}</Typography.Text>
                </div>
                <Typography.Paragraph type="secondary" className="!mb-2 text-sm">
                  {example.description}
                </Typography.Paragraph>
                <CodeBlock language="json" code={example.content} filename={`${example.id}.json`} />
                {example.notes.length > 0 && (
                  <Alert
                    type="warning"
                    showIcon
                    message="注意事项"
                    description={
                      <ul className="mb-0 list-disc pl-4">
                        {example.notes.map((note, i) => (
                          <li key={i} className="text-sm">
                            {note}
                          </li>
                        ))}
                      </ul>
                    }
                    className="mt-2"
                  />
                )}
              </div>
            ))
          )}
        </div>
      </Card>
    </div>
  );
};

export default ConfigExamples;
