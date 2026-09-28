/** 伪原创相关类型 */

export type RewriteIntensity = 'light' | 'medium' | 'heavy';

export interface RewriteOptions {
  enabled: boolean;
  intensity: RewriteIntensity;
  /** 同义词替换比例 10-50 (%) */
  synonym_ratio: number;
  /** 句子改写比例 (%) */
  sentence_ratio: number;
  paragraph_shuffle: boolean;
  rewrite_ends: boolean;
  /** 需要自然插入的关键词 */
  keywords: string[];
}

export interface RewritePreview {
  original: string;
  rewritten: string;
  options: RewriteOptions;
}
