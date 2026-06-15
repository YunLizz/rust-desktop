import React, { useEffect, useState } from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { useAIStore } from '@/stores/aiStore';
import { Plus, FolderOpen, Trash2, Clock, Key, Sparkles } from 'lucide-react';
import { clsx } from 'clsx';
import type { Project } from '@/types';

export const ProjectPage: React.FC = () => {
  const { getProjects, createProject, loadProject, deleteProject, isLoading } = useProjectStore();
  const { apiKey, setApiKey } = useAIStore();
  const [projects, setProjects] = useState<Project[]>([]);
  const [showNewProject, setShowNewProject] = useState(false);
  const [newProjectName, setNewProjectName] = useState('');
  const [showApiKey, setShowApiKey] = useState(false);
  const [tempApiKey, setTempApiKey] = useState(apiKey);

  useEffect(() => {
    loadProjectsList();
  }, []);

  const loadProjectsList = async () => {
    const list = await getProjects();
    setProjects(list);
  };

  const handleCreateProject = async () => {
    if (!newProjectName.trim()) return;
    await createProject(newProjectName.trim());
    setNewProjectName('');
    setShowNewProject(false);
    loadProjectsList();
  };

  const handleDeleteProject = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm('确定要删除这个项目吗？')) {
      await deleteProject(id);
      loadProjectsList();
    }
  };

  const handleSaveApiKey = () => {
    if (tempApiKey.trim()) {
      setApiKey(tempApiKey.trim());
      setShowApiKey(false);
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  return (
    <div className="h-full flex flex-col items-center justify-center p-8">
      {/* 标题区域 */}
      <div className="text-center mb-12">
        <h1 className="text-4xl font-bold text-[var(--color-accent-gold)] mb-2">墨韵</h1>
        <p className="text-[var(--color-text-secondary)]">AI 小说创作平台</p>
      </div>

      {/* API Key 配置 */}
      <div className="card mb-8 w-full max-w-md">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <Key size={16} className="text-[var(--color-accent-cyan)]" />
            <span className="font-medium text-[var(--color-text-primary)]">DeepSeek API</span>
          </div>
          <button
            onClick={() => setShowApiKey(!showApiKey)}
            className="text-sm text-[var(--color-accent-cyan)] hover:underline"
          >
            {apiKey ? '更改' : '配置'}
          </button>
        </div>
        {showApiKey ? (
          <div className="space-y-3">
            <input
              type="password"
              value={tempApiKey}
              onChange={(e) => setTempApiKey(e.target.value)}
              placeholder="输入你的 DeepSeek API Key"
              className="input-base"
            />
            <button onClick={handleSaveApiKey} className="btn-primary w-full">
              保存
            </button>
          </div>
        ) : (
          <p className={clsx('text-sm', apiKey ? 'text-[var(--color-success)]' : 'text-[var(--color-text-muted)]')}>
            {apiKey ? '✓ API Key 已配置' : '未配置 - 将无法使用 AI 功能'}
          </p>
        )}
      </div>

      {/* 项目列表 */}
      <div className="w-full max-w-2xl">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-[var(--color-text-primary)]">我的项目</h2>
          <button onClick={() => setShowNewProject(true)} className="btn-primary flex items-center gap-2">
            <Plus size={16} />
            新建项目
          </button>
        </div>

        {isLoading ? (
          <div className="text-center py-8 text-[var(--color-text-muted)]">加载中...</div>
        ) : projects.length === 0 ? (
          <div className="card text-center py-12">
            <FolderOpen size={48} className="mx-auto mb-4 text-[var(--color-text-muted)]" />
            <p className="text-[var(--color-text-secondary)] mb-4">还没有项目</p>
            <button onClick={() => setShowNewProject(true)} className="btn-secondary">
              创建第一个项目
            </button>
          </div>
        ) : (
          <div className="grid gap-3">
            {projects.map((project) => (
              <div
                key={project.id}
                onClick={() => loadProject(project.id)}
                className="card flex items-center gap-4 cursor-pointer hover:border-[var(--color-accent-gold)] transition-colors"
              >
                <div className="w-10 h-10 rounded-lg bg-[var(--color-bg-tertiary)] flex items-center justify-center">
                  <Sparkles size={20} className="text-[var(--color-accent-gold)]" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-medium text-[var(--color-text-primary)] truncate">
                    {project.name}
                  </h3>
                  <div className="flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
                    <Clock size={12} />
                    <span>{formatDate(project.updatedAt)}</span>
                  </div>
                </div>
                <button
                  onClick={(e) => handleDeleteProject(project.id, e)}
                  className="p-2 rounded-lg hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] hover:text-[var(--color-error)] transition-colors"
                >
                  <Trash2 size={16} />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 新建项目弹窗 */}
      {showNewProject && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card w-full max-w-md p-6">
            <h3 className="text-lg font-semibold text-[var(--color-text-primary)] mb-4">
              创建新项目
            </h3>
            <input
              type="text"
              value={newProjectName}
              onChange={(e) => setNewProjectName(e.target.value)}
              placeholder="项目名称"
              className="input-base mb-4"
              autoFocus
              onKeyDown={(e) => e.key === 'Enter' && handleCreateProject()}
            />
            <div className="flex gap-3">
              <button onClick={() => setShowNewProject(false)} className="btn-secondary flex-1">
                取消
              </button>
              <button onClick={handleCreateProject} className="btn-primary flex-1">
                创建
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
