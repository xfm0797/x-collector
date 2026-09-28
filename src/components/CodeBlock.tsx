import React from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';

export interface CodeBlockProps {
  language: string;
  code: string;
  /** 显示复制按钮 */
  copyable?: boolean;
  /** 下载文件名（提供时显示下载按钮） */
  filename?: string;
}

/** 独立代码块组件：语法高亮 + 复制 + 下载 */
const CodeBlockView: React.FC<CodeBlockProps> = ({ language, code, copyable = true, filename }) => {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // 忽略剪贴板错误
    }
  };

  const handleDownload = () => {
    const blob = new Blob([code], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename || 'config.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="group relative my-3 overflow-hidden rounded-lg">
      {(copyable || filename) && (
        <div className="absolute right-2 top-2 z-10 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
          {copyable && (
            <button
              onClick={handleCopy}
              className="rounded bg-white/10 px-2 py-0.5 text-xs text-white hover:bg-white/20"
            >
              {copied ? '已复制' : '复制'}
            </button>
          )}
          {filename && (
            <button
              onClick={handleDownload}
              className="rounded bg-white/10 px-2 py-0.5 text-xs text-white hover:bg-white/20"
            >
              下载 JSON
            </button>
          )}
        </div>
      )}
      <SyntaxHighlighter
        language={language}
        style={oneDark}
        customStyle={{ margin: 0, borderRadius: 8 }}
      >
        {code}
      </SyntaxHighlighter>
    </div>
  );
};

export default CodeBlockView;
