import React from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { useEditorStore } from '@/stores/editorStore';
import { EditorView } from '@/components/editor/EditorView';
import { OutlineView } from '@/components/editor/OutlineView';
import { TimelineView } from '@/components/visualization/TimelineView';
import { CharactersView } from '@/components/visualization/CharactersView';
import { SkillsView } from '@/components/visualization/SkillsView';
import { MapView } from '@/components/visualization/MapView';
import { NameGeneratorView } from '@/components/ai/NameGeneratorView';
import { ProjectPage } from '@/pages/ProjectPage';

export const MainContent: React.FC = () => {
  const { currentProject } = useProjectStore();
  const { currentView } = useEditorStore();

  if (!currentProject) {
    return <ProjectPage />;
  }

  const renderView = () => {
    switch (currentView) {
      case 'editor':
        return <EditorView />;
      case 'outline':
        return <OutlineView />;
      case 'timeline':
        return <TimelineView />;
      case 'characters':
        return <CharactersView />;
      case 'skills':
        return <SkillsView />;
      case 'map':
        return <MapView />;
      case 'names':
        return <NameGeneratorView />;
      default:
        return <EditorView />;
    }
  };

  return (
    <main className="flex-1 overflow-hidden bg-[var(--color-bg-primary)]">
      {renderView()}
    </main>
  );
};
