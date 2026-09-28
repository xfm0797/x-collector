import React, { useEffect, useMemo, useState } from 'react';
import { Card, Tag, Input, Collapse, Empty, Spin, Typography } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import MarkdownRenderer from '@/components/MarkdownRenderer';
import { useHelpStore } from '@/stores/helpStore';
import type { FaqItem } from '@/types/help';

const categoryLabels: Record<string, string> = {
  install: '安装',
  collect: '采集',
  publish: '发布',
  rewrite: '伪原创',
  error: '错误码',
};

/** 常见问题页面 */
const FaqPage: React.FC = () => {
  const navigate = useNavigate();
  const { faqCategories, faqs, activeFaqCategory, fetchFaqCategories, fetchFaqs, setActiveFaqCategory } = useHelpStore();
  const [keyword, setKeyword] = useState('');
  const [errorCodes, setErrorCodes] = useState<{ code: string; name: string }[]>([]);

  useEffect(() => {
    void fetchFaqCategories();
    void fetchFaqs();
    void import('@/services/helpService').then((m) => m.getErrorCodes().then(setErrorCodes).catch(() => undefined));
  }, [fetchFaqCategories, fetchFaqs]);

  useEffect(() => {
    void fetchFaqs(activeFaqCategory === 'all' ? undefined : activeFaqCategory);
  }, [activeFaqCategory, fetchFaqs]);

  const filtered = useMemo(
    () =>
      faqs.filter(
        (f: FaqItem) =>
          !keyword.trim() ||
          f.question.toLowerCase().includes(keyword.toLowerCase()) ||
          f.answer.toLowerCase().includes(keyword.toLowerCase()) ||
          f.tags.some((t) => t.toLowerCase().includes(keyword.toLowerCase())),
      ),
    [faqs, keyword],
  );

  return (
    <div className="space-y-3">
      <Card size="small" className="bg-gradient-to-r from-maple-50 to-white">
        <Typography.Title level={5} className="!mb-2">
          ❓ 常见问题
        </Typography.Title>
        <Input
          allowClear
          prefix={<SearchOutlined className="text-slate-400" />}
          placeholder="搜索问题..."
          size="large"
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
        />
        <div className="mt-3 flex flex-wrap gap-2">
          <Tag.CheckableTag checked={activeFaqCategory === 'all'} onChange={() => setActiveFaqCategory('all')}>
            全部
          </Tag.CheckableTag>
          {faqCategories.map((c) => (
            <Tag.CheckableTag key={c.id} checked={activeFaqCategory === c.id} onChange={() => setActiveFaqCategory(c.id)}>
              {c.icon} {c.name}（{c.count}）
            </Tag.CheckableTag>
          ))}
        </div>
      </Card>

      <Card size="small">
        {filtered.length === 0 ? (
          <Empty description="未找到相关问题，可尝试搜索或查看用户手册">
            <Typography.Link onClick={() => navigate('/help/manual')}>查看用户手册</Typography.Link>
          </Empty>
        ) : (
          <Collapse
            items={filtered.map((f) => ({
              key: f.id,
              label: (
                <span>
                  <span className="mr-2">❓</span>
                  {f.question}
                </span>
              ),
              children: (
                <div>
                  <MarkdownRenderer content={f.answer} />
                  {f.tags.length > 0 && (
                    <div className="mt-2 flex gap-1">
                      {f.tags.map((t) => (
                        <Tag key={t} color="blue">
                          {t}
                        </Tag>
                      ))}
                    </div>
                  )}
                </div>
              ),
            }))}
          />
        )}
      </Card>

      {activeFaqCategory === 'error' && errorCodes.length > 0 && (
        <Card title="错误码速查表" size="small">
          {errorCodes.map((e) => (
            <div key={e.code} className="mb-1 text-sm">
              <Tag color="red">{e.code}</Tag>
              <span className="text-slate-700">{e.name}</span>
            </div>
          ))}
        </Card>
      )}
    </div>
  );
};

export default FaqPage;
