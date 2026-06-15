import { create } from 'zustand';
import type { Chapter } from '@/types';

interface EditorState {
  // 当前编辑的章节
  activeChapterId: string | null;
  activeChapter: Chapter | null;

  // 编辑器状态
  content: string;
  selection: { start: number; end: number } | null;
  isDirty: boolean;

  // UI 状态
  showOutline: boolean;
  showAIPanel: boolean;
  currentView: 'editor' | 'outline' | 'timeline' | 'characters' | 'skills' | 'map' | 'names';

  // 操作
  setActiveChapter: (chapter: Chapter | null) => void;
  setContent: (content: string) => void;
  setSelection: (selection: { start: number; end: number } | null) => void;
  setShowOutline: (show: boolean) => void;
  setShowAIPanel: (show: boolean) => void;
  setCurrentView: (view: EditorState['currentView']) => void;
  markClean: () => void;
  markDirty: () => void;
}

export const useEditorStore = create<EditorState>((set, get) => ({
  activeChapterId: null,
  activeChapter: null,
  content: '',
  selection: null,
  isDirty: false,
  showOutline: true,
  showAIPanel: false,
  currentView: 'editor',

  setActiveChapter: (chapter) => {
    set({
      activeChapterId: chapter?.id || null,
      activeChapter: chapter,
      content: chapter?.content || '',
      isDirty: false,
    });
  },

  setContent: (content) => {
    set({ content, isDirty: true });
  },

  setSelection: (selection) => {
    set({ selection });
  },

  setShowOutline: (show) => {
    set({ showOutline: show });
  },

  setShowAIPanel: (show) => {
    set({ showAIPanel: show });
  },

  setCurrentView: (view) => {
    set({ currentView: view });
  },

  markClean: () => set({ isDirty: false }),
  markDirty: () => set({ isDirty: true }),
}));
