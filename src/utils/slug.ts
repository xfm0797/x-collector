/** 生成 URL/文件名安全的 slug */

export function slugify(text: string): string {
  return (
    text
      .toLowerCase()
      .replace(/[^\w\u4e00-\u9fff]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .slice(0, 80) || 'untitled'
  );
}

/** Hexo 文件名：date-title.md */
export function hexoFilename(title: string, date?: string): string {
  const d = (date || new Date().toISOString()).slice(0, 10).replace(/-/g, '');
  return `${d}-${slugify(title)}.md`;
}
