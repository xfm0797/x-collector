import React from 'react';
import { extractHeadings } from '@/utils/markdown';

interface TocSidebarProps {
  content: string;
  /** 当前激活的锚点 */
  activeId?: string;
}

/** 目录侧边栏：从 Markdown 提取标题生成 TOC */
const TocSidebar: React.FC<TocSidebarProps> = ({ content, activeId }) => {
  const headings = extractHeadings(content).filter((h) => h.level <= 3);

  if (headings.length === 0) return null;

  const scrollTo = (id: string) => {
    const el = document.getElementById(id);
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
  };

  return (
    <nav className="w-56 shrink-0 border-l border-slate-200 pl-3">
      <div className="mb-2 text-xs font-semibold uppercase text-slate-400">目录</div>
      <ul className="space-y-1 text-sm">
        {headings.map((h) => (
          <li
            key={h.id}
            className={`cursor-pointer truncate rounded px-2 py-0.5 hover:bg-slate-100 ${
              activeId === h.id ? 'bg-maple-50 text-maple-600' : 'text-slate-600'
            }`}
            style={{ paddingLeft: (h.level - 1) * 12 + 8 }}
            onClick={() => scrollTo(h.id)}
          >
            {h.text}
          </li>
        ))}
      </ul>
    </nav>
  );
};

export default TocSidebar;
