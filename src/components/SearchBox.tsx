import React, { useState } from 'react';
import { Input, List, Tag, Empty, Spin } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useHelpStore } from '@/stores/helpStore';
import type { SearchResult } from '@/types/help';

const sectionLabels: Record<string, string> = {
  manual: '手册',
  example: '示例',
  faq: 'FAQ',
};

const sectionColors: Record<string, string> = {
  manual: 'blue',
  example: 'green',
  faq: 'orange',
};

interface SearchBoxProps {
  placeholder?: string;
  onSearch?: (query: string) => void;
  /** 内嵌渲染搜索结果 */
  showResults?: boolean;
}

/** 帮助搜索框：全量搜索手册/示例/FAQ */
const SearchBox: React.FC<SearchBoxProps> = ({
  placeholder = '搜索帮助内容...',
  onSearch,
  showResults = true,
}) => {
  const navigate = useNavigate();
  const { searchHelp, searchResults, isSearching } = useHelpStore();
  const [query, setQuery] = useState('');

  const handleSearch = (value: string) => {
    setQuery(value);
    if (onSearch) {
      onSearch(value);
    } else if (value.trim()) {
      void searchHelp(value);
    }
  };

  const handleResultClick = (item: SearchResult) => {
    if (item.section === 'manual') {
      navigate(`/help/manual/${item.path}`);
    } else if (item.section === 'example') {
      navigate('/help/examples');
    } else {
      navigate('/help/faq');
    }
  };

  return (
    <div className="w-full">
      <Input
        allowClear
        prefix={<SearchOutlined className="text-slate-400" />}
        placeholder={placeholder}
        size="large"
        value={query}
        onChange={(e) => handleSearch(e.target.value)}
        onPressEnter={() => handleSearch(query)}
      />
      {showResults && query.trim() !== '' && (
        <div className="mt-2">
          {isSearching ? (
            <div className="flex justify-center py-6">
              <Spin />
            </div>
          ) : searchResults.length === 0 ? (
            <Empty description="未找到相关内容" image={Empty.PRESENTED_IMAGE_SIMPLE} />
          ) : (
            <List
              size="small"
              dataSource={searchResults}
              renderItem={(item) => (
                <List.Item
                  className="cursor-pointer hover:bg-slate-50"
                  onClick={() => handleResultClick(item)}
                >
                  <List.Item.Meta
                    title={
                      <span>
                        <Tag color={sectionColors[item.section]}>{sectionLabels[item.section]}</Tag>
                        {item.title}
                      </span>
                    }
                    description={item.snippet}
                  />
                </List.Item>
              )}
            />
          )}
        </div>
      )}
    </div>
  );
};

export default SearchBox;
