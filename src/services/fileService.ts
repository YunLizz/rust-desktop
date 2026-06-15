import {
  exists,
  mkdir,
  readTextFile,
  writeTextFile,
  remove,
  readDir,
  BaseDirectory,
} from '@tauri-apps/plugin-fs';
import type { Project, Chapter, Character, TimelineEvent, SkillNode, MapData } from '@/types';

const PROJECT_DIR = '.墨韵';
const PROJECTS_DIR = `${PROJECT_DIR}/projects`;

class FileService {
  private baseDir = BaseDirectory.Home;

  private async ensureDir(path: string): Promise<void> {
    try {
      const dirExists = await exists(path, { baseDir: this.baseDir });
      if (!dirExists) {
        await mkdir(path, { baseDir: this.baseDir, recursive: true });
      }
    } catch {
      // 目录可能已存在
    }
  }

  private async ensureProjectsDir(): Promise<void> {
    await this.ensureDir(PROJECTS_DIR);
  }

  async listProjects(): Promise<Project[]> {
    try {
      await this.ensureProjectsDir();
      const entries = await readDir(PROJECTS_DIR, { baseDir: this.baseDir });
      const projects: Project[] = [];

      for (const entry of entries) {
        if (entry.isDirectory && entry.name) {
          try {
            const projectFile = `${PROJECTS_DIR}/${entry.name}/project.json`;
            const content = await readTextFile(projectFile, { baseDir: this.baseDir });
            projects.push(JSON.parse(content));
          } catch {
            // 跳过无效项目
          }
        }
      }

      return projects.sort((a, b) => b.updatedAt - a.updatedAt);
    } catch {
      return [];
    }
  }

  async loadProject(id: string): Promise<Project> {
    const projectFile = `${PROJECTS_DIR}/${id}/project.json`;
    const content = await readTextFile(projectFile, { baseDir: this.baseDir });
    return JSON.parse(content);
  }

  async saveProject(project: Project): Promise<void> {
    await this.ensureProjectsDir();
    const projectDir = `${PROJECTS_DIR}/${project.id}`;
    await this.ensureDir(projectDir);
    const projectFile = `${projectDir}/project.json`;
    await writeTextFile(projectFile, JSON.stringify(project, null, 2), {
      baseDir: this.baseDir,
    });
  }

  async deleteProject(id: string): Promise<void> {
    const projectDir = `${PROJECTS_DIR}/${id}`;
    try {
      await remove(projectDir, { baseDir: this.baseDir, recursive: true });
    } catch {
      // 目录可能不存在
    }
  }

  async loadChapters(projectId: string): Promise<Chapter[]> {
    try {
      const file = `${PROJECTS_DIR}/${projectId}/chapters.json`;
      const content = await readTextFile(file, { baseDir: this.baseDir });
      return JSON.parse(content);
    } catch {
      return [];
    }
  }

  async saveChapters(projectId: string, chapters: Chapter[]): Promise<void> {
    const projectDir = `${PROJECTS_DIR}/${projectId}`;
    await this.ensureDir(projectDir);
    const file = `${projectDir}/chapters.json`;
    await writeTextFile(file, JSON.stringify(chapters, null, 2), {
      baseDir: this.baseDir,
    });
  }

  async loadCharacters(projectId: string): Promise<Character[]> {
    try {
      const file = `${PROJECTS_DIR}/${projectId}/characters.json`;
      const content = await readTextFile(file, { baseDir: this.baseDir });
      return JSON.parse(content);
    } catch {
      return [];
    }
  }

  async saveCharacters(projectId: string, characters: Character[]): Promise<void> {
    const projectDir = `${PROJECTS_DIR}/${projectId}`;
    await this.ensureDir(projectDir);
    const file = `${projectDir}/characters.json`;
    await writeTextFile(file, JSON.stringify(characters, null, 2), {
      baseDir: this.baseDir,
    });
  }

  async loadTimeline(projectId: string): Promise<TimelineEvent[]> {
    try {
      const file = `${PROJECTS_DIR}/${projectId}/timeline.json`;
      const content = await readTextFile(file, { baseDir: this.baseDir });
      return JSON.parse(content);
    } catch {
      return [];
    }
  }

  async saveTimeline(projectId: string, timeline: TimelineEvent[]): Promise<void> {
    const projectDir = `${PROJECTS_DIR}/${projectId}`;
    await this.ensureDir(projectDir);
    const file = `${projectDir}/timeline.json`;
    await writeTextFile(file, JSON.stringify(timeline, null, 2), {
      baseDir: this.baseDir,
    });
  }

  async loadSkills(projectId: string): Promise<SkillNode[]> {
    try {
      const file = `${PROJECTS_DIR}/${projectId}/skills.json`;
      const content = await readTextFile(file, { baseDir: this.baseDir });
      return JSON.parse(content);
    } catch {
      return [];
    }
  }

  async saveSkills(projectId: string, skills: SkillNode[]): Promise<void> {
    const projectDir = `${PROJECTS_DIR}/${projectId}`;
    await this.ensureDir(projectDir);
    const file = `${projectDir}/skills.json`;
    await writeTextFile(file, JSON.stringify(skills, null, 2), {
      baseDir: this.baseDir,
    });
  }

  async loadMapData(projectId: string): Promise<MapData> {
    try {
      const file = `${PROJECTS_DIR}/${projectId}/map.json`;
      const content = await readTextFile(file, { baseDir: this.baseDir });
      return JSON.parse(content);
    } catch {
      return { width: 800, height: 600, markers: [] };
    }
  }

  async saveMapData(projectId: string, mapData: MapData): Promise<void> {
    const projectDir = `${PROJECTS_DIR}/${projectId}`;
    await this.ensureDir(projectDir);
    const file = `${projectDir}/map.json`;
    await writeTextFile(file, JSON.stringify(mapData, null, 2), {
      baseDir: this.baseDir,
    });
  }
}

export const fileService = new FileService();
