import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Card, Input, Button, Space, Tabs, message, Tag, Typography, Descriptions, Modal } from 'antd';
import { SaveOutlined, ArrowLeftOutlined, EditOutlined, CloudUploadOutlined, SyncOutlined } from '@ant-design/icons';
import MarkdownRenderer from '@/components/MarkdownRenderer';
import PublishDialog from '@/components/PublishDialog';
import RewriteDialog from '@/components/RewriteDialog';
import RewritePreviewComponent from '@/components/RewritePreview';
import { usePublish } from '@/hooks/usePublish';
import { useRewrite } from '@/hooks/useRewrite';
import { useArticleStore } from '@/stores/articleStore';
import type { Article } from '@/types/article';
import type { RewritePreview } from '@/types/rewrite';
import { formatDateTime, timeAgo } from '@/utils/date';
import { extractExcerpt } from '@/utils/markdown';

/** 文章编辑器：Markdown 编辑 + 实时预览 + 发布/伪原创 */
const ArticleEditor: React.FC = () => {
  const { id } = useParams();
  const isNew = id === 'new';
  const navigate = useNavigate();
  const { fetchArticle, updateArticle, collectSinglePage } = useArticleStore();
  const { connections, publishArticle } = usePublish();
  const { options, fetchOptions, saveOptions, runPreview, applyRewrite } = useRewrite();

  const [article, setArticle] = useState<Article | null>(null);
  const [title, setTitle] = useState('');
  const [tags, setTags] = useState('');
  const [category, setCategory] = useState('');
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(!isNew);
  const [saving, setSaving] = useState(false);
  const [publishOpen, setPublishOpen] = useState(false);
  const [rewriteOpen, setRewriteOpen] = useState(false);
  const [previewOpen, setPreviewOpen] = useState(false);
  const [rewritePreview, setRewritePreview] = useState<RewritePreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [collectUrl, setCollectUrl] = useState('');

  useEffect(() => {
    if (!isNew && id) {
      setLoading(true);
      fetchArticle(Number(id))
        .then((a) => {
          setArticle(a);
          setTitle(a.title);
          setTags((a.tags || []).join('，'));
          setCategory(a.category || '');
          setContent(a.content_md || '');
        })
        .catch((e) => message.error(String(e)))
        .finally(() => setLoading(false));
    }
    void fetchOptions();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  const handleSave = async () => {
    if (!title.trim()) {
      message.warning('标题不能为空');
      return;
    }
    setSaving(true);
    try {
      if (isNew) {
        const { createArticle } = await import('@/services/articleService');
        const created = await createArticle({
          title: title.trim(),
          url: collectUrl || `manual-${Date.now()}`,
          content_md: content,
          category: category || undefined,
          tags: tags ? tags.split(/[,，\s]+/).filter(Boolean) : undefined,
          excerpt: content ? extractExcerpt(content) : undefined,
        });
        message.success('创建成功');
        navigate(`/articles/${created.id}`, { replace: true });
      } else {
        await updateArticle(Number(id), {
          title: title.trim(),
          content_md: content,
          category,
          tags: tags ? tags.split(/[,，\s]+/).filter(Boolean) : undefined,
          excerpt: content ? extractExcerpt(content) : undefined,
          status: 'edited',
        });
        message.success('保存成功');
      }
    } catch (e) {
      message.error(String(e));
    } finally {
      setSaving(false);
    }
  };

  const handlePublish = async (target: { publish_type: string; cms_id?: number; output_dir?: string }) => {
    if (!article) return;
    try {
      const record = await publishArticle(article.id, target as never);
      if (record.status === 'success') {
        message.success(`发布成功${record.remote_url ? `：${record.remote_url}` : ''}`);
      } else {
        message.error(`发布失败：${record.error_message}`);
      }
    } catch (e) {
      message.error(String(e));
    }
  };

  const handlePreviewRewrite = async () => {
    if (!article) return;
    setPreviewLoading(true);
    try {
      const p = await runPreview(article.id);
      setRewritePreview(p);
    } catch (e) {
      message.error(String(e));
    } finally {
      setPreviewLoading(false);
    }
  };

  const handleApplyRewrite = async () => {
    if (!article) return;
    try {
      await applyRewrite(article.id);
      const updated = await fetchArticle(article.id);
      setContent(updated.content_md || '');
      setArticle(updated);
      message.success('已应用伪原创');
      setPreviewOpen(false);
    } catch (e) {
      message.error(String(e));
    }
  };

  if (loading) return <Card loading />;

  return (
    <div className="space-y-3">
      <Card size="small">
        <div className="flex items-center justify-between">
          <Space>
            <Button icon={<ArrowLeftOutlined />} onClick={() => navigate('/articles')}>
              返回
            </Button>
            <Input
              placeholder="文章标题"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              style={{ width: 420 }}
              size="large"
              variant="borderless"
              className="font-semibold"
            />
          </Space>
          <Space>
            {article && (
              <>
                <Button icon={<SyncOutlined />} onClick={() => setPreviewOpen(true)}>
                  伪原创
                </Button>
                <Button icon={<EditOutlined />} onClick={() => setRewriteOpen(true)}>
                  改写设置
                </Button>
                <Button icon={<CloudUploadOutlined />} type="primary" onClick={() => setPublishOpen(true)}>
                  发布
                </Button>
              </>
            )}
            <Button icon={<SaveOutlined />} type="primary" loading={saving} onClick={handleSave}>
              保存
            </Button>
          </Space>
        </div>
      </Card>

      {article && (
        <Card size="small">
          <Descriptions size="small" column={4}>
            <Descriptions.Item label="来源">{article.source || '-'}</Descriptions.Item>
            <Descriptions.Item label="作者">{article.author || '-'}</Descriptions.Item>
            <Descriptions.Item label="采集时间">{timeAgo(article.collected_at)}</Descriptions.Item>
            <Descriptions.Item label="更新时间">{formatDateTime(article.updated_at)}</Descriptions.Item>
          </Descriptions>
          {article.url && article.url.startsWith('http') && (
            <Typography.Link href={article.url} target="_blank" className="text-xs">
              {article.url}
            </Typography.Link>
          )}
        </Card>
      )}

      <Card size="small">
        <div className="mb-2 grid grid-cols-2 gap-3">
          <Input
            placeholder="标签（逗号分隔）"
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            prefix={<Tag className="mr-0">标签</Tag>}
          />
          <Input placeholder="分类" value={category} onChange={(e) => setCategory(e.target.value)} />
        </div>
        {isNew && (
          <div className="mb-2">
            <Input placeholder="原文 URL（可选，用于溯源）" value={collectUrl} onChange={(e) => setCollectUrl(e.target.value)} />
          </div>
        )}
        <Tabs
          items={[
            {
              key: 'edit',
              label: '编辑',
              children: (
                <textarea
                  className="h-[560px] w-full resize-none rounded border border-slate-200 p-3 font-mono text-sm leading-6 outline-none focus:border-maple-400"
                  placeholder="支持 Markdown 格式"
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                />
              ),
            },
            {
              key: 'preview',
              label: '预览',
              children: (
                <div className="h-[560px] overflow-auto rounded border border-slate-200 bg-white p-4">
                  <MarkdownRenderer content={content || '暂无内容'} />
                </div>
              ),
            },
            {
              key: 'split',
              label: '分屏',
              children: (
                <div className="grid h-[560px] grid-cols-2 gap-2">
                  <textarea
                    className="h-full w-full resize-none rounded border border-slate-200 p-3 font-mono text-sm leading-6 outline-none focus:border-maple-400"
                    value={content}
                    onChange={(e) => setContent(e.target.value)}
                  />
                  <div className="h-full overflow-auto rounded border border-slate-200 bg-white p-4">
                    <MarkdownRenderer content={content || '暂无内容'} />
                  </div>
                </div>
              ),
            },
          ]}
        />
      </Card>

      <PublishDialog
        open={publishOpen}
        articleIds={article ? [article.id] : []}
        connections={connections}
        onCancel={() => setPublishOpen(false)}
        onSubmit={handlePublish}
      />

      {options && (
        <RewriteDialog
          open={rewriteOpen}
          options={options}
          onCancel={() => setRewriteOpen(false)}
          onSubmit={saveOptions}
        />
      )}

      <Modal
        title="伪原创预览"
        open={previewOpen}
        onCancel={() => setPreviewOpen(false)}
        width={960}
        footer={[
          <Button key="preview" onClick={handlePreviewRewrite} loading={previewLoading}>
            预览改写
          </Button>,
          <Button key="apply" type="primary" onClick={handleApplyRewrite}>
            应用到文章
          </Button>,
        ]}
      >
        <RewritePreviewComponent preview={rewritePreview} loading={previewLoading} />
      </Modal>
    </div>
  );
};

export default ArticleEditor;
