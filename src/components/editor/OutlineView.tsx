import React, { useState } from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { useEditorStore } from '@/stores/editorStore';
import { Plus, GripVertical, Trash2, Edit2, Check, X } from 'lucide-react';
import { clsx } from 'clsx';

export const OutlineView: React.FC = () => {
  const { chapters, createChapter, updateChapter, deleteChapter, reorderChapters } = useProjectStore();
  const { setActiveChapter, setCurrentView } = useEditorStore();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState('');

  const handleCreateChapter = async () => {
    const chapter = await createChapter('新章节');
    setEditingId(chapter.id);
    setEditingTitle(chapter.title);
  };

  const handleStartEdit = (id: string, title: string) => {
    setEditingId(id);
    setEditingTitle(title);
  };

  const handleSaveEdit = async () => {
    if (!editingId) return;
    const chapter = chapters.find((c) => c.id === editingId);
    if (chapter) {
      await updateChapter({ ...chapter, title: editingTitle });
    }
    setEditingId(null);
    setEditingTitle('');
  };

  const handleCancelEdit = () => {
    setEditingId(null);
    setEditingTitle('');
  };

  const handleDelete = async (id: string) => {
    if (confirm('确定要删除这个章节吗？')) {
      await deleteChapter(id);
    }
  };

  const handleSelectChapter = (chapter: typeof chapters[0]) => {
    setActiveChapter(chapter);
    setCurrentView('editor');
  };

  const rootChapters = chapters
    .filter((c) => !c.parentId)
    .sort((a, b) => a.order - b.order);

  const getChildren = (parentId: string) =>
    chapters.filter((c) => c.parentId === parentId).sort((a, b) => a.order - b.order);

  const renderChapter = (chapter: typeof chapters[0], level: number = 0) => {
    const children = getChildren(chapter.id);
    const isEditing = editingId === chapter.id;

    return (
      <div key={chapter.id}>
        <div
          className={clsx(
            'flex items-center gap-2 px-3 py-2 rounded-lg group cursor-pointer transition-colors',
            'hover:bg-[var(--color-bg-tertiary)]'
          )}
          style={{ paddingLeft: `${12 + level * 20}px` }}
        >
          <GripVertical size={14} className="text-[var(--color-text-muted)] cursor-grab" />
          {isEditing ? (
            <>
              <input
                type="text"
                value={editingTitle}
                onChange={(e) => setEditingTitle(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleSaveEdit();
                  if (e.key === 'Escape') handleCancelEdit();
                }}
                className="flex-1 px-2 py-1 bg-[var(--color-bg-primary)] border border-[var(--color-border)] rounded text-sm"
                autoFocus
              />
              <button onClick={handleSaveEdit} className="p-1 text-[var(--color-success)]">
                <Check size={14} />
              </button>
              <button onClick={handleCancelEdit} className="p-1 text-[var(--color-text-muted)]">
                <X size={14} />
              </button>
            </>
          ) : (
            <>
              <span
                className="flex-1 text-sm text-[var(--color-text-primary)] truncate"
                onClick={() => handleSelectChapter(chapter)}
              >
                {chapter.title || '未命名章节'}
              </span>
              <button
                onClick={() => handleStartEdit(chapter.id, chapter.title)}
                className="p-1 text-[var(--color-text-muted)] opacity-0 group-hover:opacity-100 hover:text-[var(--color-accent-cyan)]"
              >
                <Edit2 size={12} />
              </button>
              <button
                onClick={() => handleDelete(chapter.id)}
                className="p-1 text-[var(--color-text-muted)] opacity-0 group-hover:opacity-100 hover:text-[var(--color-error)]"
              >
                <Trash2 size={12} />
              </button>
            </>
          )}
        </div>
        {children.map((child) => renderChapter(child, level + 1))}
      </div>
    );
  };

  return (
    <div className="h-full flex flex-col p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-[var(--color-text-primary)]">大纲编辑</h2>
        <button onClick={handleCreateChapter} className="btn-primary flex items-center gap-2">
          <Plus size={16} />
          新建章节
        </button>
      </div>

      <div className="flex-1 overflow-auto">
        {rootChapters.length === 0 ? (
          <div className="text-center py-12 text-[var(--color-text-muted)]">
            <p>暂无章节</p>
            <button onClick={handleCreateChapter} className="btn-secondary mt-4">
              创建第一个章节
            </button>
          </div>
        ) : (
          <div className="space-y-1">{rootChapters.map((chapter) => renderChapter(chapter))}</div>
        )}
      </div>
    </div>
  );
};
