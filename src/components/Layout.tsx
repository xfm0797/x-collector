import React from 'react';
import { Outlet, useNavigate } from 'react-router-dom';
import { Layout as AntLayout, Button, Input, Tooltip } from 'antd';
import { QuestionCircleOutlined, SettingOutlined } from '@ant-design/icons';
import Sidebar from './Sidebar';

const { Header, Sider, Content } = AntLayout;

/** 应用整体布局：顶部标题栏 + 侧边栏导航 + 主内容区 */
const Layout: React.FC = () => {
  const navigate = useNavigate();

  return (
    <AntLayout className="h-screen">
      <Header className="flex items-center justify-between border-b border-slate-200 bg-white px-4" style={{ height: 52 }}>
        <div className="flex items-center gap-3">
          <span className="text-2xl">🍃</span>
          <span className="text-base font-semibold tracking-wide text-slate-800">枫铃采集器</span>
          <span className="rounded bg-maple-50 px-2 py-0.5 text-xs text-maple-600">v0.0.1</span>
        </div>
        <div className="flex items-center gap-1">
          <Tooltip title="帮助中心">
            <Button
              type="text"
              icon={<QuestionCircleOutlined />}
              onClick={() => navigate('/help')}
            />
          </Tooltip>
          <Tooltip title="设置">
            <Button
              type="text"
              icon={<SettingOutlined />}
              onClick={() => navigate('/settings')}
            />
          </Tooltip>
        </div>
      </Header>
      <AntLayout>
        <Sider width={200} theme="light" className="border-r border-slate-200">
          <Sidebar />
        </Sider>
        <Content className="overflow-auto bg-slate-50 p-4">
          <Outlet />
        </Content>
      </AntLayout>
    </AntLayout>
  );
};

export default Layout;
