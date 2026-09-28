# 枫铃采集器 (x-collector)

<p align="center">
  <b>关键词采集 · 内容清洗 · 伪原创 · 多平台发布</b>
</p>

![version](https://img.shields.io/badge/version-0.0.1-orange)
![tauri](https://img.shields.io/badge/Tauri-2.x-blue)
![license](https://img.shields.io/badge/license-All%20Rights%20Reserved-lightgrey)

基于 **Tauri 2 + React 18 + Rust + SQLite** 的桌面采集与发布工具，支持关键词采集、RSS/网页/站点地图采集、内容清洗（含伪原创）、Hexo 导出与 WordPress / Typecho / Z-Blog 自动发布。

## 功能特性

| 模块 | 功能 |
|------|------|
| 📊 仪表盘 | 文章 / 采集 / 发布统计总览 |
| 📝 文章库 | Markdown 编辑器（实时预览）、搜索、批量管理 |
| 🔍 关键词采集 | 百度 / Google / Bing / 搜狗，标题/全文/精确匹配，采集深度 1-10 页 |
| 📡 采集源 | RSS 订阅、单页采集、sitemap 批量发现、自定义 CSS 选择器 |
| ✨ 内容清洗 | 广告/导航/侧边栏移除、链接绝对化、GBK 编码识别、HTML→Markdown |
| 🔄 伪原创 | 同义词替换（120+ 词库）、句子改写、段落重排、首尾段重写、自定义词库 |
| 🛡 反爬 | UA 轮换、完整浏览器请求头、Cookie 携带、随机延迟、指数退避 |
| 🧹 去重 | URL 哈希 + 标题相似度（编辑距离 ≥85%）+ 内容指纹 |
| 📤 发布 | Hexo 导出、WordPress / Typecho / Z-Blog XML-RPC 自动发布、发布记录追踪 |
| ❓ 帮助 | 用户手册（8 章）、配置示例（6 套）、FAQ、全文搜索、错误码速查 |

## 快速开始

### 环境要求

- Node.js ≥ 18（推荐 20）
- Rust ≥ 1.80（含 cargo）
- 平台依赖：Windows 需要 WebView2（Win11 内置）

### 安装与运行

```bash
# 安装前端依赖
npm install

# 开发模式（自动热更新）
npm run tauri dev

# 打包发布
npm run tauri build
```

### 三步上手

1. 「关键词采集」→ 添加关键词 → 点击「采集」；
2. 「文章库」查看 / 编辑采集结果；
3. 「发布管理」→ 选择文章 → 发布到 Hexo 或 CMS。

详细指引见应用内「帮助中心 → 用户手册」。

## 项目结构

```
x-collector/
├── src/                 # React 前端（页面/组件/状态/服务）
├── src-tauri/           # Rust 后端（命令/采集引擎/发布/帮助/数据库）
├── docs/                # 用户手册、配置示例、FAQ（嵌入二进制，离线可用）
└── CHANGELOG.md
```

## 技术栈

- **前端**：React 18 · TypeScript · Ant Design 5 · Tailwind CSS 3 · Zustand 4 · react-markdown 9
- **后端**：Rust · Tauri 2 · SQLite (rusqlite) · reqwest · scraper · rust-embed

## 开发

```bash
npm run dev              # 仅启动前端
cargo check              # Rust 类型检查（src-tauri）
npm run build            # 前端生产构建
```

## 版权

Copyright © 2026 XFM. All rights reserved.

- 作者：XFM
- 主页：<https://github.com/xfm0797>
- 问题反馈：<https://github.com/xfm0797/x-collector/issues>

> 请遵守目标站点的 robots 协议与当地法律法规，合理控制采集频率，仅将采集内容用于合法用途。
