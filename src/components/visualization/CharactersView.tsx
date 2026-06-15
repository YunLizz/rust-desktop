import React, { useState, useRef, useEffect } from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { Plus, Trash2, User, Users } from 'lucide-react';
import { clsx } from 'clsx';
import type { Character, Relationship } from '@/types';

const RELATION_COLORS: Record<Relationship['type'], string> = {
  family: '#3fb950',
  friend: '#58a6ff',
  lover: '#f85149',
  enemy: '#d29922',
  neutral: '#8b949e',
};

export const CharactersView: React.FC = () => {
  const { characters, createCharacter, updateCharacter, deleteCharacter } = useProjectStore();
  const [showForm, setShowForm] = useState(false);
  const [editingChar, setEditingChar] = useState<Character | null>(null);
  const [formData, setFormData] = useState({ name: '', description: '' });
  const [selectedChar, setSelectedChar] = useState<Character | null>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const handleAdd = async () => {
    if (!formData.name) return;
    await createCharacter(formData.name, formData.description);
    setShowForm(false);
    setFormData({ name: '', description: '' });
  };

  const handleEdit = (char: Character) => {
    setEditingChar(char);
    setFormData({ name: char.name, description: char.description });
  };

  const handleUpdate = async () => {
    if (!editingChar || !formData.name) return;
    await updateCharacter({ ...editingChar, name: formData.name, description: formData.description });
    setEditingChar(null);
    setFormData({ name: '', description: '' });
  };

  const handleDelete = async (id: string) => {
    if (confirm('确定要删除这个角色吗？')) {
      await deleteCharacter(id);
      if (selectedChar?.id === id) setSelectedChar(null);
    }
  };

  // 绘制关系图
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * window.devicePixelRatio;
    canvas.height = rect.height * window.devicePixelRatio;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);

    ctx.clearRect(0, 0, rect.width, rect.height);

    const centerX = rect.width / 2;
    const centerY = rect.height / 2;
    const radius = Math.min(centerX, centerY) - 60;

    // 绘制连线
    characters.forEach((char, i) => {
      const angle = (i / characters.length) * Math.PI * 2 - Math.PI / 2;
      const charX = centerX + Math.cos(angle) * radius;
      const charY = centerY + Math.sin(angle) * radius;

      char.relationships.forEach((rel) => {
        const targetIndex = characters.findIndex((c) => c.id === rel.targetId);
        if (targetIndex === -1) return;

        const targetAngle = (targetIndex / characters.length) * Math.PI * 2 - Math.PI / 2;
        const targetX = centerX + Math.cos(targetAngle) * radius;
        const targetY = centerY + Math.sin(targetAngle) * radius;

        ctx.beginPath();
        ctx.moveTo(charX, charY);
        ctx.lineTo(targetX, targetY);
        ctx.strokeStyle = RELATION_COLORS[rel.type];
        ctx.lineWidth = 2;
        ctx.globalAlpha = 0.5;
        ctx.stroke();
        ctx.globalAlpha = 1;
      });
    });

    // 绘制节点
    characters.forEach((char, i) => {
      const angle = (i / characters.length) * Math.PI * 2 - Math.PI / 2;
      const x = centerX + Math.cos(angle) * radius;
      const y = centerY + Math.sin(angle) * radius;

      // 圆圈
      ctx.beginPath();
      ctx.arc(x, y, 30, 0, Math.PI * 2);
      ctx.fillStyle = selectedChar?.id === char.id ? '#d4a574' : '#21262d';
      ctx.fill();
      ctx.strokeStyle = '#d4a574';
      ctx.lineWidth = 2;
      ctx.stroke();

      // 文字
      ctx.fillStyle = '#e6edf3';
      ctx.font = '12px Microsoft YaHei';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(char.name.substring(0, 4), x, y);
    });
  }, [characters, selectedChar]);

  const selectedCharDetails = characters.find((c) => c.id === selectedChar?.id);

  return (
    <div className="h-full flex p-6 gap-6">
      {/* 角色列表 */}
      <div className="w-64 flex flex-col">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-[var(--color-text-primary)]">人物关系</h2>
          <button onClick={() => setShowForm(true)} className="btn-primary p-2">
            <Plus size={16} />
          </button>
        </div>

        {/* 添加/编辑表单 */}
        {(showForm || editingChar) && (
          <div className="card mb-4 p-3">
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder="角色名称"
              className="input-base mb-2"
            />
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="角色描述"
              className="input-base mb-2 min-h-[60px]"
            />
            <div className="flex gap-2">
              <button onClick={editingChar ? handleUpdate : handleAdd} className="btn-primary flex-1 text-sm">
                {editingChar ? '保存' : '添加'}
              </button>
              <button
                onClick={() => {
                  setShowForm(false);
                  setEditingChar(null);
                  setFormData({ name: '', description: '' });
                }}
                className="btn-secondary flex-1 text-sm"
              >
                取消
              </button>
            </div>
          </div>
        )}

        <div className="flex-1 overflow-auto space-y-2">
          {characters.map((char) => (
            <div
              key={char.id}
              onClick={() => setSelectedChar(char)}
              className={clsx(
                'flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors',
                selectedChar?.id === char.id
                  ? 'bg-[var(--color-bg-tertiary)] border border-[var(--color-accent-gold)]'
                  : 'bg-[var(--color-bg-secondary)] border border-transparent hover:border-[var(--color-border)]'
              )}
            >
              <div className="w-10 h-10 rounded-full bg-[var(--color-bg-tertiary)] flex items-center justify-center">
                <User size={20} className="text-[var(--color-accent-gold)]" />
              </div>
              <div className="flex-1 min-w-0">
                <h3 className="font-medium text-[var(--color-text-primary)] truncate">{char.name}</h3>
                <p className="text-xs text-[var(--color-text-muted)] truncate">
                  {char.relationships.length} 个关系
                </p>
              </div>
              <div className="flex gap-1">
                <button onClick={() => handleEdit(char)} className="p-1 text-[var(--color-text-muted)]">
                  <Users size={14} />
                </button>
                <button
                  onClick={() => handleDelete(char.id)}
                  className="p-1 text-[var(--color-text-muted)] hover:text-[var(--color-error)]"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          ))}

          {characters.length === 0 && (
            <div className="text-center py-8 text-[var(--color-text-muted)]">
              <User size={32} className="mx-auto mb-2" />
              <p className="text-sm">暂无角色</p>
            </div>
          )}
        </div>
      </div>

      {/* 关系图 */}
      <div className="flex-1 card flex flex-col">
        <h3 className="font-medium text-[var(--color-text-primary)] mb-3">关系图</h3>
        <canvas ref={canvasRef} className="flex-1 bg-[var(--color-bg-primary)] rounded-lg" />
      </div>

      {/* 选中角色详情 */}
      {selectedCharDetails && (
        <div className="w-64 card">
          <h3 className="font-medium text-[var(--color-text-primary)] mb-3">{selectedCharDetails.name}</h3>
          <p className="text-sm text-[var(--color-text-secondary)] mb-4">
            {selectedCharDetails.description || '暂无描述'}
          </p>
          <h4 className="text-sm font-medium text-[var(--color-text-primary)] mb-2">关系</h4>
          <div className="space-y-2">
            {selectedCharDetails.relationships.map((rel, i) => {
              const target = characters.find((c) => c.id === rel.targetId);
              return (
                <div key={i} className="flex items-center gap-2 text-sm">
                  <div
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: RELATION_COLORS[rel.type] }}
                  />
                  <span className="text-[var(--color-text-secondary)]">{target?.name || '未知'}</span>
                  <span className="text-[var(--color-text-muted)]">
                    ({rel.type === 'family' ? '家人' : rel.type === 'friend' ? '朋友' : rel.type === 'lover' ? '恋人' : rel.type === 'enemy' ? '敌人' : '中立'})
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};
