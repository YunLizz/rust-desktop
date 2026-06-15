import { create } from 'zustand';
import type { ChatMessage, GenerateOptions, PlotBranch, ValidationResult } from '@/types';
import { deepseekService } from '@/services/deepseek';

interface AIState {
  apiKey: string;
  isProcessing: boolean;
  error: string | null;
  conversationHistory: ChatMessage[];

  setApiKey: (key: string) => void;
  clearApiKey: () => void;
  isConfigured: () => boolean;

  // AI 生成功能
  generateContinuation: (
    currentText: string,
    context?: string
  ) => Promise<string>;

  generateSummary: (content: string) => Promise<string>;

  generatePlotTree: (
    outline: string,
    branches?: number
  ) => Promise<PlotBranch[]>;

  generateNames: (
    type: 'character' | 'place' | 'faction',
    style?: string,
    count?: number
  ) => Promise<string[]>;

  validateTimeline: (
    events: { timestamp: string; description: string }[],
    worldContext: string
  ) => Promise<ValidationResult[]>;

  validateWorldConsistency: (
    newContent: string,
    existingContent: string[]
  ) => Promise<ValidationResult[]>;

  // 对话
  sendMessage: (message: string) => Promise<string>;

  clearHistory: () => void;
  clearError: () => void;
}

export const useAIStore = create<AIState>((set, get) => ({
  apiKey: '',
  isProcessing: false,
  error: null,
  conversationHistory: [],

  setApiKey: (key: string) => {
    deepseekService.setApiKey(key);
    set({ apiKey: key });
    // 尝试保存到本地存储
    try {
      localStorage.setItem('deepseek_api_key', key);
    } catch {}
  },

  clearApiKey: () => {
    deepseekService.setApiKey('');
    set({ apiKey: '' });
    try {
      localStorage.removeItem('deepseek_api_key');
    } catch {}
  },

  isConfigured: () => {
    const { apiKey } = get();
    return !!apiKey;
  },

  generateContinuation: async (currentText: string, context?: string) => {
    if (!get().isConfigured()) {
      throw new Error('请先设置 DeepSeek API Key');
    }
    set({ isProcessing: true, error: null });
    try {
      const result = await deepseekService.generateContinuation(currentText, context);
      set({ isProcessing: false });
      return result;
    } catch (error) {
      set({ error: (error as Error).message, isProcessing: false });
      throw error;
    }
  },

  generateSummary: async (content: string) => {
    if (!get().isConfigured()) {
      throw new Error('请先设置 DeepSeek API Key');
    }
    set({ isProcessing: true, error: null });
    try {
      const result = await deepseekService.generateSummary(content);
      set({ isProcessing: false });
      return result;
    } catch (error) {
      set({ error: (error as Error).message, isProcessing: false });
      throw error;
    }
  },

  generatePlotTree: async (outline: string, branches = 3) => {
    if (!get().isConfigured()) {
      throw new Error('请先设置 DeepSeek API Key');
    }
    set({ isProcessing: true, error: null });
    try {
      const result = await deepseekService.generatePlotTree(outline, branches);
      set({ isProcessing: false });
      return result;
    } catch (error) {
      set({ error: (error as Error).message, isProcessing: false });
      throw error;
    }
  },

  generateNames: async (type, style = 'default', count = 10) => {
    if (!get().isConfigured()) {
      throw new Error('请先设置 DeepSeek API Key');
    }
    set({ isProcessing: true, error: null });
    try {
      const result = await deepseekService.generateNames(type, style, count);
      set({ isProcessing: false });
      return result;
    } catch (error) {
      set({ error: (error as Error).message, isProcessing: false });
      throw error;
    }
  },

  validateTimeline: async (events, worldContext) => {
    if (!get().isConfigured()) {
      throw new Error('请先设置 DeepSeek API Key');
    }
    set({ isProcessing: true, error: null });
    try {
      const result = await deepseekService.validateTimeline(events, worldContext);
      set({ isProcessing: false });
      return result;
    } catch (error) {
      set({ error: (error as Error).message, isProcessing: false });
      throw error;
    }
  },

  validateWorldConsistency: async (newContent, existingContent) => {
    if (!get().isConfigured()) {
      throw new Error('请先设置 DeepSeek API Key');
    }
    set({ isProcessing: true, error: null });
    try {
      const result = await deepseekService.validateWorldConsistency(newContent, existingContent);
      set({ isProcessing: false });
      return result;
    } catch (error) {
      set({ error: (error as Error).message, isProcessing: false });
      throw error;
    }
  },

  sendMessage: async (message: string) => {
    if (!get().isConfigured()) {
      throw new Error('请先设置 DeepSeek API Key');
    }
    set({ isProcessing: true, error: null });

    const { conversationHistory } = get();
    const newHistory: ChatMessage[] = [...conversationHistory, { role: 'user', content: message }];

    set({ conversationHistory: newHistory });

    try {
      const response = await deepseekService.chat(newHistory);
      const updatedHistory: ChatMessage[] = [
        ...newHistory,
        { role: 'assistant', content: response },
      ];
      set({ conversationHistory: updatedHistory, isProcessing: false });
      return response;
    } catch (error) {
      set({ error: (error as Error).message, isProcessing: false });
      throw error;
    }
  },

  clearHistory: () => set({ conversationHistory: [] }),
  clearError: () => set({ error: null }),
}));
