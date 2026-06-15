import React from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { useEditorStore } from '@/stores/editorStore';
import {
  FolderOpen,
  FileText,
  Clock,
  Users,
  GitBranch,
  Map,
  PenTool,
  Sparkles,
  ChevronRight,
  ChevronDown,
  Plus,
  MoreVertical,
  Settings,
} from 'lucide-react';
import { clsx } from 'clsx';

interface NavItemProps {
  icon: React.ReactNode;
  label: string;
  active?: boolean;
  onClick: () => void;
  indent?: number;
}

const NavItem: React.FC<NavItemProps> = ({ icon, label, active, onClick, indent = 0 }) => (
  <button
    onClick={onClick}
    className={clsx(
      'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left transition-all duration-200',
      'hover:bg-[var(--color-bg-tertiary)]',
      active ? 'bg-[var(--color-bg-tertiary)] text-[var(--color-accent-gold)]' : 'text-[var(--color-text-secondary)]'
    )}
    style={{ paddingLeft: `${12 + indent * 16}px` }}
  >
    {icon}
    <span className="flex-1 truncate text-sm">{label}</span>
  </button>
);

interface SectionProps {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
  defaultOpen?: boolean;
}

const Section: React.FC<SectionProps> = ({ title, icon, children, defaultOpen = true }) => {
  const [isOpen, setIsOpen] = React.useState(defaultOpen);

  return (
    <div className="mb-2">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center gap-2 px-3 py-2 text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)] transition-colors"
      >
        {isOpen ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
        {icon}
        <span className="text-xs font-medium uppercase tracking-wide">{title}</span>
      </button>
      {isOpen && <div className="mt-1">{children}</div>}
    </div>
  );
};

export const Sidebar: React.FC = () => {
  const { currentProject, chapters, characters, timeline, setActiveChapter } = useProjectStore();
  const { currentView, setCurrentView, activeChapterId } = useEditorStore();
  const [contextMenu, setContextMenu] = React.useState<string | null>(null);

  if (!currentProject) {
    return (
      <aside className="w-64 h-full flex flex-col bg-[var(--color-bg-secondary)] border-r border-[var(--color-border)]">
        <div className="p-4 border-b border-[var(--color-border)]">
          <h1 className="text-lg font-bold text-[var(--color-accent-gold)]">墨韵</h1>
          <p className="text-xs text-[var(--color-text-muted)] mt-1">AI 小说创作平台</p>
        </div>
        <div className="flex-1 flex items-center justify-center text-[var(--color-text-muted)] text-sm">
          选择或创建一个项目
        </div>
      </aside>
    );
  }

  return (
    <aside className="w-64 h-full flex flex-col bg-[var(--color-bg-secondary)] border-r border-[var(--color-border)]">
      {/* Header */}
      <div className="p-4 border-b border-[var(--color-border)]">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <FolderOpen size={18} className="text-[var(--color-accent-gold)]" />
            <h2 className="font-semibold text-[var(--color-text-primary)] truncate">
              {currentProject.name}
            </h2>
          </div>
          <button
            onClick={() => setContextMenu(contextMenu ? null : 'project')}
            className="p-1 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)]"
          >
            <MoreVertical size={16} />
          </button>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-2">
        <Section title="编辑" icon={<FileText size={14} />}>
          <NavItem
            icon={<FileText size={14} />}
            label="正文编辑"
            active={currentView === 'editor'}
            onClick={() => setCurrentView('editor')}
          />
          <NavItem
            icon={<GitBranch size={14} />}
            label="大纲"
            active={currentView === 'outline'}
            onClick={() => setCurrentView('outline')}
          />
        </Section>

        <Section title="创作辅助" icon={<Sparkles size={14} />}>
          <NavItem
            icon={<Clock size={14} />}
            label="时间轴"
            active={currentView === 'timeline'}
            onClick={() => setCurrentView('timeline')}
          />
          <NavItem
            icon={<Users size={14} />}
            label="人物关系"
            active={currentView === 'characters'}
            onClick={() => setCurrentView('characters')}
          />
          <NavItem
            icon={<GitBranch size={14} />}
            label="技能树"
            active={currentView === 'skills'}
            onClick={() => setCurrentView('skills')}
          />
          <NavItem
            icon={<Map size={14} />}
            label="地图"
            active={currentView === 'map'}
            onClick={() => setCurrentView('map')}
          />
          <NavItem
            icon={<PenTool size={14} />}
            label="起名机"
            active={currentView === 'names'}
            onClick={() => setCurrentView('names')}
          />
        </Section>

        {/* Chapters */}
        {chapters.length > 0 && (
          <Section title="章节" icon={<FileText size={14} />}>
            {chapters
              .filter((c) => !c.parentId)
              .sort((a, b) => a.order - b.order)
              .map((chapter) => (
                <NavItem
                  key={chapter.id}
                  icon={<FileText size={14} />}
                  label={chapter.title || '未命名章节'}
                  active={activeChapterId === chapter.id}
                  onClick={() => {
                    setActiveChapter(chapter);
                    setCurrentView('editor');
                  }}
                />
              ))}
          </Section>
        )}
      </nav>

      {/* Footer */}
      <div className="p-2 border-t border-[var(--color-border)]">
        <button className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-[var(--color-text-muted)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text-secondary)] transition-colors">
          <Settings size={14} />
          <span className="text-sm">设置</span>
        </button>
      </div>
    </aside>
  );
};
