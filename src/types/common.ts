/** 通用类型定义 */

/** 分页数据 */
export interface PageData<T> {
  list: T[];
  total: number;
  page: number;
  page_size: number;
}

/** 采集进度事件载荷 */
export interface CollectProgress {
  stage: string;
  current: number;
  total: number;
  message: string;
}

/** 采集日志事件载荷 */
export interface CollectLogEntry {
  status: 'success' | 'failed' | 'info';
  url?: string;
  message: string;
}

/** 采集结果摘要 */
export interface CollectSummary {
  found: number;
  collected: number;
  skipped: number;
  failed: number;
}

/** 单项操作结果 */
export interface OpResult {
  success: boolean;
  message: string;
}
