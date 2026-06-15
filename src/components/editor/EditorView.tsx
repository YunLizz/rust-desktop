import React, { useCallback, useEffect, useRef } from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { useEditorStore } from '@/stores/editorStore';
import { useAIStore } from '@/stores/aiStore';
import {
  Bold,
  Italic,
  Heading1,
  Heading2,
  List,
  ListOrdered,
  Quote,
  Undo,
  Redo,
  Save,
  Sparkles,
  FilePlus,
} from 'lucide-react';
import { clsx } from 'clsx';

export const EditorView: React.FC = () => {
  const { activeChapter, setActiveChapter, updateChapter, createChapter, saveProject } = useProjectStore();
  const { content, setContent, selection, setSelection, isDirty } = useEditorStore();
  const { generateContinuation, isProcessing, isConfigured } = useAIStore();
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [generatedText, setGeneratedText] = React.useState<string | null>(null);

  useEffect(() => {
    if (activeChapter) {
      setContent(activeChapter.content);
    }
  }, [activeChapter?.id]);

  const handleSave = useCallback(async () => {
    if (!activeChapter) return;
    await updateChapter({ ...activeChapter, content });
    setContent(content);
  }, [activeChapter, content, updateChapter]);

  const handleContentChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setContent(e.target.value);
  };

  const handleSelect = (e: React.SyntheticEvent<HTMLTextAreaElement>) => {
    const target = e.target as HTMLTextAreaElement;
    setSelection({ start: target.selectionStart, end: target.selectionEnd });
  };

  const insertText = (before: string, after: string = '') => {
    if (!textareaRef.current || !selection) return;
    const start = selection.start;
    const end = selection.end;
    const selectedText = content.substring(start, end);
    const newContent = content.substring(0, start) + before + selectedText + after + content.substring(end);
    setContent(newContent);
    setTimeout(() => {
      textareaRef.current?.focus();
      textareaRef.current?.setSelectionRange(start + before.length, end + before.length);
    }, 0);
  };

  const handleToolbarAction = (action: string) => {
    switch (action) {
      case 'bold':
        insertText('**', '**');
        break;
      case 'italic':
        insertText('*', '*');
        break;
      case 'h1':
        insertText('# ');
        break;
      case 'h2':
        insertText('## ');
        break;
      case 'list':
        insertText('- ');
        break;
      case 'ordered':
        insertText('1. ');
        break;
      case 'quote':
        insertText('> ');
        break;
    }
  };

  const handleGenerate = async () => {
    if (!isConfigured()) {
      alert('请先配置 DeepSeek API Key');
      return;
    }
    try {
      const result = await generateContinuation(content);
      setGeneratedText(result);
    } catch (error) {
      alert((error as Error).message);
    }
  };

  const handleApplyGenerated = () => {
    if (generatedText) {
      setContent(content + '\n\n' + generatedText);
      setGeneratedText(null);
    }
  };

  if (!activeChapter) {
    return (
      <div className="h-full flex flex-col items-center justify-center">
        <div className="text-center">
          <FilePlus size={64} className="mx-auto mb-4 text-[var(--color-text-muted)]" />
          <h2 className="text-xl font-semibold text-[var(--color-text-primary)] mb-2">
            选择或创建章节
          </h2>
          <p className="text-[var(--color-text-secondary)] mb-4">
            在左侧边栏选择章节，或创建新章节开始写作
          </p>
          <button
            onClick={async () => {
              const chapter = await createChapter('新章节');
              setActiveChapter(chapter);
            }}
            className="btn-primary"
          >
            创建新章节
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
        <div className="flex items-center gap-1">
          <button
            onClick={() => handleToolbarAction('bold')}
            className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]"
            title="加粗"
          >
            <Bold size={16} />
          </button>
          <button
            onClick={() => handleToolbarAction('italic')}
            className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]"
            title="斜体"
          >
            <Italic size={16} />
          </button>
          <div className="w-px h-6 bg-[var(--color-border)] mx-2" />
          <button
            onClick={() => handleToolbarAction('h1')}
            className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]"
            title="标题1"
          >
            <Heading1 size={16} />
          </button>
          <button
            onClick={() => handleToolbarAction('h2')}
            className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]"
            title="标题2"
          >
            <Heading2 size={16} />
          </button>
          <div className="w-px h-6 bg-[var(--color-border)] mx-2" />
          <button
            onClick={() => handleToolbarAction('list')}
            className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]"
            title="无序列表"
          >
            <List size={16} />
          </button>
          <button
            onClick={() => handleToolbarAction('ordered')}
            className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]"
            title="有序列表"
          >
            <ListOrdered size={16} />
          </button>
          <button
            onClick={() => handleToolbarAction('quote')}
            className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]"
            title="引用"
          >
            <Quote size={16} />
          </button>
          <div className="w-px h-6 bg-[var(--color-border)] mx-2" />
          <button className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]">
            <Undo size={16} />
          </button>
          <button className="p-2 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]">
            <Redo size={16} />
          </button>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleGenerate}
            disabled={isProcessing || !isConfigured()}
            className={clsx(
              'btn-secondary flex items-center gap-2',
              isProcessing && 'opacity-50'
            )}
          >
            <Sparkles size={16} />
            AI 续写
          </button>
          <button
            onClick={handleSave}
            disabled={!isDirty}
            className={clsx('btn-secondary flex items-center gap-2', !isDirty && 'opacity-50')}
          >
            <Save size={16} />
            保存
          </button>
        </div>
      </div>

      {/* 编辑器 */}
      <div className="flex-1 flex overflow-hidden">
        <div className="flex-1 p-6 overflow-auto">
          <textarea
            ref={textareaRef}
            value={content}
            onChange={handleContentChange}
            onSelect={handleSelect}
            placeholder="开始写作..."
            className="w-full h-full bg-transparent border-none resize-none focus:outline-none text-[var(--color-text-primary)] leading-relaxed"
            style={{ fontSize: '16px', lineHeight: '1.8' }}
          />
        </div>

        {/* AI 生成结果面板 */}
        {generatedText && (
          <div className="w-96 border-l border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4 overflow-auto">
            <h3 className="font-semibold text-[var(--color-text-primary)] mb-3 flex items-center gap-2">
              <Sparkles size={16} className="text-[var(--color-accent-gold)]" />
              AI 生成结果
            </h3>
            <div className="p-4 rounded-lg bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] text-sm whitespace-pre-wrap mb-4">
              {generatedText}
            </div>
            <div className="flex gap-2">
              <button onClick={handleApplyGenerated} className="btn-primary flex-1">
                应用
              </button>
              <button onClick={() => setGeneratedText(null)} className="btn-secondary flex-1">
                取消
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
