import { create } from 'zustand';
import type { Project, Chapter, Character, TimelineEvent, SkillNode, MapData } from '@/types';
import { fileService } from '@/services/fileService';
import { v4 as uuidv4 } from 'uuid';

interface ProjectState {
  // 数据
  currentProject: Project | null;
  chapters: Chapter[];
  characters: Character[];
  timeline: TimelineEvent[];
  skills: SkillNode[];
  mapData: MapData;
  activeChapter: Chapter | null;

  // 状态
  isLoading: boolean;
  error: string | null;

  // 项目操作
  createProject: (name: string, description?: string) => Promise<void>;
  loadProject: (id: string) => Promise<void>;
  saveProject: () => Promise<void>;
  deleteProject: (id: string) => Promise<void>;
  getProjects: () => Promise<Project[]>;

  // 章节操作
  createChapter: (title: string, parentId?: string) => Promise<Chapter>;
  updateChapter: (chapter: Chapter) => Promise<void>;
  deleteChapter: (id: string) => Promise<void>;
  reorderChapters: (chapterIds: string[]) => Promise<void>;
  setActiveChapter: (chapter: Chapter | null) => void;

  // 人物操作
  createCharacter: (name: string, description: string) => Promise<Character>;
  updateCharacter: (character: Character) => Promise<void>;
  deleteCharacter: (id: string) => Promise<void>;

  // 时间轴操作
  addTimelineEvent: (event: Omit<TimelineEvent, 'id'>) => Promise<void>;
  updateTimelineEvent: (event: TimelineEvent) => Promise<void>;
  deleteTimelineEvent: (id: string) => Promise<void>;

  // 技能树操作
  addSkillNode: (node: Omit<SkillNode, 'id'>) => Promise<void>;
  updateSkillNode: (node: SkillNode) => Promise<void>;
  deleteSkillNode: (id: string) => Promise<void>;

  // 地图操作
  updateMapData: (data: MapData) => Promise<void>;

  // 清理
  clearError: () => void;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  currentProject: null,
  chapters: [],
  characters: [],
  timeline: [],
  skills: [],
  mapData: { width: 800, height: 600, markers: [] },
  activeChapter: null,
  isLoading: false,
  error: null,

  getProjects: async () => {
    try {
      const projects = await fileService.listProjects();
      return projects;
    } catch (error) {
      set({ error: (error as Error).message });
      return [];
    }
  },

  createProject: async (name: string, description?: string) => {
    set({ isLoading: true, error: null });
    try {
      const project: Project = {
        id: uuidv4(),
        name,
        description,
        createdAt: Date.now(),
        updatedAt: Date.now(),
        settings: {},
      };
      await fileService.saveProject(project);
      set({ currentProject: project, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  loadProject: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const project = await fileService.loadProject(id);
      const chapters = await fileService.loadChapters(id);
      const characters = await fileService.loadCharacters(id);
      const timeline = await fileService.loadTimeline(id);
      const skills = await fileService.loadSkills(id);
      const mapData = await fileService.loadMapData(id);

      set({
        currentProject: project,
        chapters,
        characters,
        timeline,
        skills,
        mapData,
        isLoading: false,
      });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  saveProject: async () => {
    const { currentProject, chapters, characters, timeline, skills, mapData } = get();
    if (!currentProject) return;

    set({ isLoading: true, error: null });
    try {
      const updatedProject = { ...currentProject, updatedAt: Date.now() };
      await fileService.saveProject(updatedProject);
      await fileService.saveChapters(currentProject.id, chapters);
      await fileService.saveCharacters(currentProject.id, characters);
      await fileService.saveTimeline(currentProject.id, timeline);
      await fileService.saveSkills(currentProject.id, skills);
      await fileService.saveMapData(currentProject.id, mapData);
      set({ currentProject: updatedProject, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  deleteProject: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await fileService.deleteProject(id);
      const { currentProject } = get();
      if (currentProject?.id === id) {
        set({ currentProject: null, chapters: [], characters: [], timeline: [] });
      }
      set({ isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  // 章节操作
  createChapter: async (title: string, parentId?: string) => {
    const { currentProject, chapters } = get();
    if (!currentProject) throw new Error('No project loaded');

    const maxOrder = chapters
      .filter((c) => c.parentId === parentId)
      .reduce((max, c) => Math.max(max, c.order), -1);

    const chapter: Chapter = {
      id: uuidv4(),
      title,
      content: '',
      order: maxOrder + 1,
      parentId,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };

    set({ chapters: [...chapters, chapter] });
    await get().saveProject();
    return chapter;
  },

  updateChapter: async (chapter: Chapter) => {
    const { chapters } = get();
    const updated = { ...chapter, updatedAt: Date.now() };
    set({ chapters: chapters.map((c) => (c.id === chapter.id ? updated : c)) });
    await get().saveProject();
  },

  deleteChapter: async (id: string) => {
    const { chapters } = get();
    // 删除章节及其子章节
    const idsToDelete = new Set<string>();
    const collectIds = (chapterId: string) => {
      idsToDelete.add(chapterId);
      chapters
        .filter((c) => c.parentId === chapterId)
        .forEach((c) => collectIds(c.id));
    };
    collectIds(id);
    set({ chapters: chapters.filter((c) => !idsToDelete.has(c.id)) });
    await get().saveProject();
  },

  reorderChapters: async (chapterIds: string[]) => {
    const { chapters } = get();
    const reordered = chapters.map((c) => {
      const newOrder = chapterIds.indexOf(c.id);
      return newOrder >= 0 ? { ...c, order: newOrder } : c;
    });
    set({ chapters: reordered });
    await get().saveProject();
  },

  setActiveChapter: (chapter) => {
    set({ activeChapter: chapter });
  },

  // 人物操作
  createCharacter: async (name: string, description: string) => {
    const { characters } = get();
    const character: Character = {
      id: uuidv4(),
      name,
      description,
      relationships: [],
    };
    set({ characters: [...characters, character] });
    await get().saveProject();
    return character;
  },

  updateCharacter: async (character: Character) => {
    const { characters } = get();
    set({ characters: characters.map((c) => (c.id === character.id ? character : c)) });
    await get().saveProject();
  },

  deleteCharacter: async (id: string) => {
    const { characters } = get();
    set({ characters: characters.filter((c) => c.id !== id) });
    await get().saveProject();
  },

  // 时间轴操作
  addTimelineEvent: async (event) => {
    const { timeline } = get();
    const newEvent: TimelineEvent = { ...event, id: uuidv4() };
    set({ timeline: [...timeline, newEvent] });
    await get().saveProject();
  },

  updateTimelineEvent: async (event) => {
    const { timeline } = get();
    set({ timeline: timeline.map((e) => (e.id === event.id ? event : e)) });
    await get().saveProject();
  },

  deleteTimelineEvent: async (id: string) => {
    const { timeline } = get();
    set({ timeline: timeline.filter((e) => e.id !== id) });
    await get().saveProject();
  },

  // 技能树操作
  addSkillNode: async (node) => {
    const { skills } = get();
    const newNode: SkillNode = { ...node, id: uuidv4() };
    set({ skills: [...skills, newNode] });
    await get().saveProject();
  },

  updateSkillNode: async (node) => {
    const { skills } = get();
    set({ skills: skills.map((n) => (n.id === node.id ? node : n)) });
    await get().saveProject();
  },

  deleteSkillNode: async (id: string) => {
    const { skills } = get();
    set({ skills: skills.filter((n) => n.id !== id) });
    await get().saveProject();
  },

  // 地图操作
  updateMapData: async (data) => {
    set({ mapData: data });
    await get().saveProject();
  },

  clearError: () => set({ error: null }),
}));
