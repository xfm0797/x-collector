import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

/** Markdown 渲染器：支持代码高亮、表格、图片 */
const MarkdownRenderer: React.FC<MarkdownRendererProps> = ({ content, className }) => {
  return (
    <div className={`markdown-body text-[14px] leading-7 text-slate-800 ${className || ''}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h1: ({ children }) => <h1 className="mt-6 mb-3 text-2xl font-bold">{children}</h1>,
          h2: ({ children }) => <h2 className="mt-5 mb-2 text-xl font-semibold">{children}</h2>,
          h3: ({ children }) => <h3 className="mt-4 mb-2 text-lg font-semibold">{children}</h3>,
          p: ({ children }) => <p className="my-2">{children}</p>,
          ul: ({ children }) => <ul className="my-2 list-disc pl-6">{children}</ul>,
          ol: ({ children }) => <ol className="my-2 list-decimal pl-6">{children}</ol>,
          blockquote: ({ children }) => (
            <blockquote className="my-2 border-l-4 border-maple-300 bg-maple-50 px-3 py-1 text-slate-600">
              {children}
            </blockquote>
          ),
          table: ({ children }) => (
            <div className="my-3 overflow-x-auto">
              <table className="w-full border-collapse text-sm">{children}</table>
            </div>
          ),
          tr: ({ children }) => <tr className="border-b border-slate-200 even:bg-slate-50">{children}</tr>,
          th: ({ children }) => (
            <th className="border border-slate-300 bg-slate-100 px-2 py-1 text-left font-semibold">{children}</th>
          ),
          td: ({ children }) => <td className="border border-slate-300 px-2 py-1">{children}</td>,
          a: ({ children, href }) => (
            <a href={href} target="_blank" rel="noreferrer" className="text-maple-600 hover:underline">
              {children}
            </a>
          ),
          img: ({ src, alt }) => (
            <img src={src as string} alt={alt || ''} className="my-2 max-w-full rounded" />
          ),
          code: (props) => {
            const { className: cls, children } = props as { className?: string; children?: React.ReactNode };
            const match = /language-(\w+)/.exec(cls || '');
            const text = String(children || '').replace(/\n$/, '');
            // 无语言标记且不含换行 → 行内代码
            if (!match && !text.includes('\n')) {
              return <code className="rounded bg-slate-100 px-1.5 py-0.5 text-[13px] text-rose-600">{text}</code>;
            }
            return <CodeBlock language={match?.[1] || 'text'} code={text} />;
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

/** 代码块：语法高亮 + 一键复制 */
export const CodeBlock: React.FC<{ language: string; code: string }> = ({ language, code }) => {
  const [copied, setCopied] = React.useState(false);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // 剪贴板不可用时忽略
    }
  };

  return (
    <div className="group relative my-3">
      <button
        onClick={copy}
        className="absolute right-2 top-2 z-10 rounded bg-white/10 px-2 py-0.5 text-xs text-white opacity-0 transition-opacity group-hover:opacity-100"
      >
        {copied ? '已复制' : '复制'}
      </button>
      <SyntaxHighlighter language={language} style={oneDark} showLineNumbers={false} wrapLongLines={false}>
        {code}
      </SyntaxHighlighter>
    </div>
  );
};

export default MarkdownRenderer;
