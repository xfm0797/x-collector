# 发布配置

## 6.1 Hexo 导出

1. 在发布弹窗中输入导出目录（如 `C:/blog/source/_posts`），或提前在「设置 → 发布」配置默认目录。
2. 自动生成 Front-matter：`title` / `date` / `categories` / `tags` / `source_url`。
3. 文件命名：`YYYYMMDD-标题.md`，与 Hexo 约定一致。
4. 导出后执行 `hexo generate` 即可发布。

## 6.2 WordPress 发布

1. 「发布管理 → CMS 连接」添加连接，类型选择 WordPress。
2. 站点地址填写根域名，API 路径通常为 `/xmlrpc.php`。
3. **强烈建议使用应用密码**：WordPress 后台「用户 → 个人资料 → 应用密码」生成。
4. 点击「测试」验证连通性，然后即可单篇或批量发布。

## 6.3 Typecho 发布

与 WordPress 类似，API 路径为 `/action/xmlrpc`，使用登录用户名 + 密码。默认分类将映射到 Typecho 分类。

## 6.4 Z-Blog 发布

API 路径为 `/zb_system/xml-rpc/index.php`，需在 Z-Blog 后台启用「XML-RPC 发布协议」插件。

## 6.5 发布状态追踪

- **发布记录**页面查看每次发布的成功 / 失败与远程地址；
- 失败记录会保留错误信息，方便排查；
- 文章状态自动流转：collected → edited → published / failed。
