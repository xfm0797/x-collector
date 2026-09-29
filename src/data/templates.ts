/**
 * 内置模板库：采集源模板 / 清洗规则模板 / CMS 配置模板
 * 选择器与后端 site_adapters 保持一致
 */

import type { CollectSourceInput } from '@/types/keyword';
import type { CmsConnectionInput } from '@/types/publish';

// ── 采集源模板 ─────────────────────────────────────────────

export interface SourceTemplate {
  key: string;
  name: string;
  category: '社区博客' | '新闻资讯' | 'RSS 订阅';
  description: string;
  /** 应用到表单的部分字段 */
  values: Partial<CollectSourceInput>;
}

export const sourceTemplates: SourceTemplate[] = [
  {
    key: 'zhihu-column',
    name: '知乎专栏/回答',
    category: '社区博客',
    description: '正文选择器已适配知乎专栏页面，自动清理答题卡片等冗余元素',
    values: {
      name: '知乎专栏采集',
      source_type: 'webpage',
      selector_content: 'div.Post-RichTextContainer, article',
      selector_title: 'h1.Post-Title',
      selector_author: 'div.AuthorInfo-name',
      remove_selectors: ['div.Post-SideActions', 'div.Comments-container', 'div.ContentItem-actions'],
    },
  },
  {
    key: 'csdn-blog',
    name: 'CSDN / 博客园',
    category: '社区博客',
    description: '兼容 CSDN 与博客园正文容器，移除推荐栏与付费提示',
    values: {
      name: 'CSDN 博客采集',
      source_type: 'webpage',
      selector_content: 'div#content_views, div.article_content, div#cnblogs_post_body',
      selector_title: 'h1.title, #activity-name',
      selector_author: 'div.blog-content-box .follow-nick-name, #blog_user_info',
      remove_selectors: ['div.recommend-box', 'div.hide-article-box', 'div.template-platform', 'div.comment-box'],
    },
  },
  {
    key: 'juejin-post',
    name: '掘金文章',
    category: '社区博客',
    description: '掘金 markdown 正文容器，清理侧边目录与推荐',
    values: {
      name: '掘金文章采集',
      source_type: 'webpage',
      selector_content: 'div.markdown-body, article',
      selector_title: 'h1.article-title',
      remove_selectors: ['div.article-suspended-panel', 'div.recommended-area', 'div.comment-list'],
    },
  },
  {
    key: 'wechat-mp',
    name: '微信公众号文章',
    category: '社区博客',
    description: '公众号正文容器，移除二维码与小程序卡片',
    values: {
      name: '公众号文章采集',
      source_type: 'webpage',
      selector_content: 'div#js_content, div.rich_media_content',
      selector_title: '#activity-name',
      selector_author: '#js_name',
      remove_selectors: ['div.qr_code_pc', 'div.wx_mmb_qrcode', 'mp-common-profile'],
    },
  },
  {
    key: 'ruanyf-rss',
    name: '阮一峰科技爱好者周刊',
    category: 'RSS 订阅',
    description: '经典 RSS 订阅源示例，无需配置选择器',
    values: {
      name: '阮一峰周刊',
      url: 'https://www.ruanyifeng.com/blog/atom.xml',
      source_type: 'rss',
      group_name: '技术周刊',
    },
  },
  {
    key: 'sspai-rss',
    name: '少数派 RSS',
    category: 'RSS 订阅',
    description: '少数派主站 RSS 订阅源',
    values: {
      name: '少数派',
      url: 'https://sspai.com/feed',
      source_type: 'rss',
      group_name: '科技资讯',
    },
  },
  {
    key: 'infoq-rss',
    name: 'InfoQ 中国 RSS',
    category: 'RSS 订阅',
    description: 'InfoQ 中文站 RSS 订阅源',
    values: {
      name: 'InfoQ 中国',
      url: 'https://www.infoq.cn/feed',
      source_type: 'rss',
      group_name: '科技资讯',
    },
  },
  {
    key: 'sitemap-generic',
    name: '站点地图批量采集',
    category: '新闻资讯',
    description: '通过 sitemap.xml 批量发现站点全部文章链接（单次最多 30 篇）',
    values: {
      name: '站点地图采集',
      url: 'https://example.com/sitemap.xml',
      source_type: 'sitemap',
    },
  },
  {
    key: 'generic-article',
    name: '通用文章页（智能提取）',
    category: '新闻资讯',
    description: '不配置选择器，由引擎按文本密度智能识别正文（类 Readability）',
    values: {
      name: '通用文章采集',
      source_type: 'webpage',
      selector_content: undefined,
    },
  },
];

// ── 清洗规则模板 ───────────────────────────────────────────

export interface CleaningTemplate {
  key: string;
  name: string;
  description: string;
  removeSelectors: string[];
}

export const cleaningTemplates: CleaningTemplate[] = [
  {
    key: 'clean-ads',
    name: '广告与推广清理',
    description: '移除常见广告位、推广横幅、悬浮按钮',
    removeSelectors: [
      'div.ad, div.adsbygoogle, ins.adsbygoogle',
      'div[class*="banner"], div[class*="promo"]',
      'div[class*="float"], div[class*="popup"]',
    ],
  },
  {
    key: 'clean-related',
    name: '相关推荐清理',
    description: '移除相关文章、推荐阅读、热门列表等非正文区块',
    removeSelectors: [
      'div[class*="related"], div[class*="recommend"]',
      'div[class*="hot"], div[class*="rank"]',
      'aside, section[class*="sidebar"]',
    ],
  },
  {
    key: 'clean-comment',
    name: '评论与互动清理',
    description: '移除评论区、点赞分享栏、打赏组件',
    removeSelectors: [
      'div[class*="comment"], div[class*="Comments"]',
      'div[class*="share"], div[class*="like"], div[class*="reward"]',
      'div[class*="actions"]',
    ],
  },
  {
    key: 'clean-nav',
    name: '导航与页脚清理',
    description: '移除面包屑、导航栏、页脚版权区块',
    removeSelectors: ['nav, header, footer', 'div[class*="breadcrumb"], div[class*="crumb"]', 'div[class*="copyright"]'],
  },
  {
    key: 'clean-full',
    name: '深度清理（全量规则）',
    description: '合并以上全部规则，最大程度保留纯正文',
    removeSelectors: [
      'div.ad, div.adsbygoogle, ins.adsbygoogle',
      'div[class*="banner"], div[class*="promo"]',
      'div[class*="related"], div[class*="recommend"]',
      'div[class*="comment"], div[class*="Comments"]',
      'div[class*="share"], div[class*="reward"]',
      'aside, nav, footer',
      'div[class*="breadcrumb"], div[class*="copyright"]',
    ],
  },
];

// ── CMS 配置模板 ───────────────────────────────────────────

export interface CmsTemplate {
  key: string;
  name: string;
  description: string;
  values: Partial<CmsConnectionInput>;
}

export const cmsTemplates: CmsTemplate[] = [
  {
    key: 'wordpress',
    name: 'WordPress',
    description: '标准 XML-RPC 接口，建议在后台生成「应用密码」后填入',
    values: { name: '我的 WordPress 博客', cms_type: 'wordpress', api_path: '/xmlrpc.php', default_status: 'publish' },
  },
  {
    key: 'typecho',
    name: 'Typecho',
    description: 'XML-RPC 位于 /action/xmlrpc，需在后台开启 XML-RPC 扩展',
    values: { name: '我的 Typecho 博客', cms_type: 'typecho', api_path: '/action/xmlrpc', default_status: 'publish' },
  },
  {
    key: 'zblog',
    name: 'Z-Blog',
    description: 'Z-BlogPHP 的 XML-RPC 接口，需在后台启用 API 模块',
    values: { name: '我的 Z-Blog 博客', cms_type: 'zblog', api_path: '/zb_system/xml-rpc/index.php', default_status: 'publish' },
  },
  {
    key: 'draft-first',
    name: 'WordPress（草稿模式）',
    description: '先保存为草稿人工审核，再正式发布',
    values: { name: 'WordPress 草稿箱', cms_type: 'wordpress', api_path: '/xmlrpc.php', default_status: 'draft' },
  },
];
