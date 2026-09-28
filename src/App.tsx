import React from 'react';
import { HashRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import Layout from '@/components/Layout';
import Dashboard from '@/pages/Dashboard';
import ArticleList from '@/pages/ArticleList';
import ArticleEditor from '@/pages/ArticleEditor';
import KeywordCollect from '@/pages/KeywordCollect';
import SourceManager from '@/pages/SourceManager';
import PublishManager from '@/pages/PublishManager';
import HelpCenter from '@/pages/HelpCenter';
import UserManual from '@/pages/UserManual';
import ConfigExamples from '@/pages/ConfigExamples';
import FaqPage from '@/pages/FaqPage';
import Settings from '@/pages/Settings';

/** 根组件：路由与 Antd 中文配置 */
const App: React.FC = () => {
  return (
    <ConfigProvider
      locale={zhCN}
      theme={{
        token: {
          colorPrimary: '#f97316',
          borderRadius: 6,
        },
      }}
    >
      <HashRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<Dashboard />} />
            <Route path="/articles" element={<ArticleList />} />
            <Route path="/articles/:id" element={<ArticleEditor />} />
            <Route path="/keywords" element={<KeywordCollect />} />
            <Route path="/sources" element={<SourceManager />} />
            <Route path="/publish" element={<PublishManager />} />
            <Route path="/help" element={<HelpCenter />} />
            <Route path="/help/manual" element={<UserManual />} />
            <Route path="/help/manual/:section" element={<UserManual />} />
            <Route path="/help/examples" element={<ConfigExamples />} />
            <Route path="/help/faq" element={<FaqPage />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
        </Routes>
      </HashRouter>
    </ConfigProvider>
  );
};

export default App;
