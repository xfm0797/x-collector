/** 帮助系统类型定义 */

/** 手册目录节点 */
export interface ManualSection {
  id: string;
  title: string;
  path: string;
  children?: ManualSection[];
  level: number;
}

export interface NavItem {
  title: string;
  path: string;
}

/** 手册章节内容 */
export interface ManualContent {
  path: string;
  title: string;
  content: string;
  prev: NavItem | null;
  next: NavItem | null;
}

/** 配置示例 */
export interface ConfigExample {
  id: string;
  platform: string;
  title: string;
  description: string;
  content: string;
  notes: string[];
}

/** FAQ 分类 */
export interface FaqCategory {
  id: string;
  name: string;
  icon: string;
  count: number;
}

/** FAQ 条目 */
export interface FaqItem {
  id: string;
  category: string;
  question: string;
  answer: string;
  tags: string[];
  related: string[];
}

/** 搜索结果 */
export interface SearchResult {
  section: string;
  title: string;
  snippet: string;
  path: string;
  relevance: number;
}

/** 快速开始引导 */
export interface QuickStartGuide {
  steps: QuickStartStep[];
}

export interface QuickStartStep {
  step: number;
  title: string;
  description: string;
  action: string;
}

/** 快捷键条目 */
export interface ShortcutItem {
  key: string;
  description: string;
  category: string;
}

/** 错误码说明 */
export interface ErrorCode {
  code: string;
  name: string;
  description: string;
  causes: string[];
  solutions: string[];
}

/** 更新信息 */
export interface UpdateInfo {
  has_update: boolean;
  latest_version: string;
  current_version: string;
  download_url: string;
  changelog: string;
}
