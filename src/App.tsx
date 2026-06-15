import React, { useEffect } from 'react';
import { Sidebar } from '@/components/layout/Sidebar';
import { MainContent } from '@/components/layout/MainContent';
import { StatusBar } from '@/components/layout/StatusBar';
import { useAIStore } from '@/stores/aiStore';

function App() {
  const { setApiKey } = useAIStore();

  // 加载保存的 API Key
  useEffect(() => {
    try {
      const savedKey = localStorage.getItem('deepseek_api_key');
      if (savedKey) {
        setApiKey(savedKey);
      }
    } catch {
      // localStorage 可能不可用
    }
  }, []);

  return (
    <div className="h-screen flex flex-col bg-[var(--color-bg-primary)]">
      <div className="flex-1 flex overflow-hidden">
        <Sidebar />
        <MainContent />
      </div>
      <StatusBar />
    </div>
  );
}

export default App;
