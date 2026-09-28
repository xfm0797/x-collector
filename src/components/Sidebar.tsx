import React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { Menu } from 'antd';
import {
  DashboardOutlined,
  FileTextOutlined,
  SearchOutlined,
  WifiOutlined,
  CloudUploadOutlined,
  QuestionCircleOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import type { MenuProps } from 'antd';

const menuItems: MenuProps['items'] = [
  { key: '/', icon: <DashboardOutlined />, label: '仪表盘' },
  { key: '/articles', icon: <FileTextOutlined />, label: '文章库' },
  { key: '/keywords', icon: <SearchOutlined />, label: '关键词采集' },
  { key: '/sources', icon: <WifiOutlined />, label: '采集源' },
  { key: '/publish', icon: <CloudUploadOutlined />, label: '发布管理' },
  { key: '/help', icon: <QuestionCircleOutlined />, label: '帮助中心' },
  { key: '/settings', icon: <SettingOutlined />, label: '设置' },
];

/** 侧边栏导航 */
const Sidebar: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();

  // 帮助中心子页面也高亮"帮助中心"
  const selectedKey = location.pathname.startsWith('/help') ? '/help' : location.pathname;

  const onClick: MenuProps['onClick'] = ({ key }) => {
    navigate(key as string);
  };

  return (
    <Menu
      mode="inline"
      selectedKeys={[selectedKey]}
      items={menuItems}
      onClick={onClick}
      className="h-full border-r-0"
    />
  );
};

export default Sidebar;
