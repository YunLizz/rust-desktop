import React from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { useEditorStore } from '@/stores/editorStore';
import { useAIStore } from '@/stores/aiStore';
import { Cloud, CloudOff, Save, Loader } from 'lucide-react';
import { clsx } from 'clsx';

export const StatusBar: React.FC = () => {
  const { currentProject, chapters } = useProjectStore();
  const { content, isDirty } = useEditorStore();
  const { isProcessing, isConfigured } = useAIStore();

  const wordCount = content.trim().length;
  const charCount = content.length;
  const chapterCount = chapters.length;

  return (
    <footer className="h-8 flex items-center justify-between px-4 bg-[var(--color-bg-secondary)] border-t border-[var(--color-border)] text-xs text-[var(--color-text-muted)]">
      <div className="flex items-center gap-4">
        {currentProject && (
          <>
            <span>章节: {chapterCount}</span>
            <span>|</span>
            <span>字数: {wordCount.toLocaleString()}</span>
            <span>|</span>
            <span>字符: {charCount.toLocaleString()}</span>
          </>
        )}
      </div>

      <div className="flex items-center gap-4">
        {isDirty && (
          <span className="flex items-center gap-1 text-[var(--color-warning)]">
            <Save size={12} />
            未保存
          </span>
        )}

        {isProcessing && (
          <span className="flex items-center gap-1 text-[var(--color-accent-cyan)]">
            <Loader size={12} className="animate-spin" />
            AI 处理中
          </span>
        )}

        <span
          className={clsx(
            'flex items-center gap-1',
            isConfigured() ? 'text-[var(--color-success)]' : 'text-[var(--color-text-muted)]'
          )}
        >
          {isConfigured() ? <Cloud size={12} /> : <CloudOff size={12} />}
          {isConfigured() ? 'API 已连接' : 'API 未配置'}
        </span>
      </div>
    </footer>
  );
};
