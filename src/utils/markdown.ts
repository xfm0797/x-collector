/** Markdown 工具函数 */

/** 从 Markdown 提取纯文本摘要 */
export function extractExcerpt(md: string, maxLength = 120): string {
  const text = md
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/!\[.*?\]\(.*?\)/g, ' ')
    .replace(/\[([^\]]*)\]\(.*?\)/g, '$1')
    .replace(/[#>*_`~|-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  return text.length > maxLength ? `${text.slice(0, maxLength)}...` : text;
}

/** 从 Markdown 提取所有标题（用于目录侧边栏） */
export function extractHeadings(md: string): { level: number; text: string; id: string }[] {
  const headings: { level: number; text: string; id: string }[] = [];
  let inCode = false;
  for (const line of md.split('\n')) {
    if (line.trim().startsWith('```')) {
      inCode = !inCode;
      continue;
    }
    if (inCode) continue;
    const m = /^(#{1,4})\s+(.+)$/.exec(line);
    if (m) {
      const text = m[2].trim();
      headings.push({ level: m[1].length, text, id: toHeadingId(text) });
    }
  }
  return headings;
}

/** 标题转锚点 ID（与 react-markdown rehype-slug 规则保持一致的简化版） */
export function toHeadingId(text: string): string {
  return encodeURIComponent(
    text
      .toLowerCase()
      .replace(/[^\w\u4e00-\u9fff\- ]/g, '')
      .replace(/ /g, '-'),
  );
}

/** 统计 Markdown 字数 */
export function countWords(md: string): number {
  const text = md.replace(/[#>*`~\-!\[\]()]/g, ' ');
  const cjk = (text.match(/[\u4e00-\u9fff]/g) || []).length;
  const words = (text.replace(/[\u4e00-\u9fff]/g, ' ').match(/[a-zA-Z0-9]+/g) || []).length;
  return cjk + words;
}
