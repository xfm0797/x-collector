/** Hexo Front-matter 工具（前端预览用，实际导出由 Rust 端完成） */

export interface FrontMatterOptions {
  title: string;
  date: string;
  tags: string[];
  categories: string[];
  sourceUrl?: string;
  author?: string;
  includeExcerpt?: boolean;
  excerpt?: string;
}

export function buildFrontMatter(opts: FrontMatterOptions): string {
  const lines: string[] = ['---'];
  lines.push(`title: ${escapeYaml(opts.title)}`);
  lines.push(`date: ${opts.date}`);
  if (opts.author) lines.push(`author: ${escapeYaml(opts.author)}`);
  if (opts.categories.length) {
    lines.push('categories:');
    opts.categories.forEach((c) => lines.push(`  - ${escapeYaml(c)}`));
  }
  if (opts.tags.length) {
    lines.push('tags:');
    opts.tags.forEach((t) => lines.push(`  - ${escapeYaml(t)}`));
  }
  if (opts.includeExcerpt && opts.excerpt) {
    lines.push(`excerpt: ${escapeYaml(opts.excerpt.slice(0, 100))}`);
  }
  if (opts.sourceUrl) lines.push(`source_url: ${opts.sourceUrl}`);
  lines.push('---');
  return lines.join('\n');
}

function escapeYaml(s: string): string {
  if (/[:#{}\[\]&*!|>'"%@`]/.test(s) || /^\s|\s$/.test(s)) {
    return `"${s.replace(/"/g, '\\"')}"`;
  }
  return s;
}
