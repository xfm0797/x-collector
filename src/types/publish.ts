/** 发布相关类型 */

export type CmsType = 'wordpress' | 'typecho' | 'zblog' | 'custom';
export type PublishType = 'hexo' | 'wordpress' | 'typecho' | 'zblog' | 'custom';

export interface CmsConnection {
  id: number;
  name: string;
  cms_type: CmsType;
  site_url: string;
  api_path?: string;
  username: string;
  default_category?: string;
  default_status: string;
  tag_mapping?: string | null;
  enabled: number;
  last_used?: string;
  created_at?: string;
  /** 不返回密码明文，仅编辑时回填 */
  has_password?: boolean;
}

export interface CmsConnectionInput {
  id?: number;
  name: string;
  cms_type: CmsType;
  site_url: string;
  api_path?: string;
  username: string;
  password?: string;
  default_category?: string;
  default_status?: string;
  tag_mapping?: Record<string, string>;
  enabled?: number;
}

export interface PublishRecord {
  id: number;
  article_id: number;
  article_title?: string;
  cms_id?: number | null;
  cms_name?: string;
  publish_type: PublishType;
  status: 'success' | 'failed' | 'pending';
  remote_id?: string;
  remote_url?: string;
  error_message?: string;
  published_at?: string;
  created_at?: string;
}

/** 发布目标：Hexo 导出或某个 CMS 连接 */
export interface PublishTarget {
  publish_type: PublishType;
  cms_id?: number;
  /** Hexo 导出目录（publish_type=hexo 时使用，留空使用设置中的默认目录） */
  output_dir?: string;
}

export interface HexoExportResult {
  files: string[];
  failed: number;
}
