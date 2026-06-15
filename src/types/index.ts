// 项目管理类型
export interface Project {
  id: string;
  name: string;
  description?: string;
  createdAt: number;
  updatedAt: number;
  settings: ProjectSettings;
}

export interface ProjectSettings {
  apiKey?: string;
  theme?: 'dark' | 'light';
}

// 章节类型
export interface Chapter {
  id: string;
  title: string;
  content: string;
  summary?: string;
  order: number;
  parentId?: string;
  createdAt: number;
  updatedAt: number;
}

// 目录项
export interface OutlineItem {
  id: string;
  title: string;
  order: number;
  parentId?: string;
  children?: OutlineItem[];
}

// 时间轴事件
export interface TimelineEvent {
  id: string;
  title: string;
  description: string;
  timestamp: string;
  chapterId?: string;
  characters?: string[];
  type: 'main' | 'branch' | 'custom';
}

// 人物
export interface Character {
  id: string;
  name: string;
  description: string;
  avatar?: string;
  attributes?: Record<string, string>;
  relationships: Relationship[];
}

export interface Relationship {
  targetId: string;
  type: 'family' | 'friend' | 'lover' | 'enemy' | 'neutral';
  description?: string;
}

// 技能树节点
export interface SkillNode {
  id: string;
  name: string;
  description: string;
  parentId?: string;
  level: number;
  position?: { x: number; y: number };
}

// 地图标记
export interface MapMarker {
  id: string;
  name: string;
  x: number;
  y: number;
  type: 'city' | 'region' | 'landmark' | 'route';
  description?: string;
  connections?: string[];
}

export interface MapData {
  width: number;
  height: number;
  markers: MapMarker[];
  background?: string;
}

// AI 相关类型
export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface GenerateOptions {
  model?: string;
  temperature?: number;
  maxTokens?: number;
}

export interface PlotBranch {
  id: string;
  description: string;
  consequences: string[];
  probability?: number;
}

export interface ValidationResult {
  type: 'error' | 'warning';
  message: string;
  location?: string;
  suggestion?: string;
}

// DeepSeek API 响应
export interface DeepSeekResponse {
  id: string;
  choices: {
    message: {
      content: string;
    };
    finish_reason: string;
  }[];
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}
