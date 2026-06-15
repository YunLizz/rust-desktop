import React, { useState, useRef, useEffect } from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { Plus, Trash2, GitBranch } from 'lucide-react';
import { clsx } from 'clsx';
import type { SkillNode } from '@/types';

export const SkillsView: React.FC = () => {
  const { skills, addSkillNode, updateSkillNode, deleteSkillNode } = useProjectStore();
  const [showForm, setShowForm] = useState(false);
  const [editingNode, setEditingNode] = useState<SkillNode | null>(null);
  const [formData, setFormData] = useState<Partial<SkillNode>>({
    name: '',
    description: '',
    level: 0,
  });
  const [selectedNode, setSelectedNode] = useState<SkillNode | null>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const handleAdd = async () => {
    if (!formData.name) return;
    await addSkillNode(formData as Omit<SkillNode, 'id'>);
    setShowForm(false);
    setFormData({ name: '', description: '', level: 0 });
  };

  const handleUpdate = async () => {
    if (!editingNode) return;
    await updateSkillNode({ ...editingNode, ...formData });
    setEditingNode(null);
    setFormData({ name: '', description: '', level: 0 });
  };

  const handleEdit = (node: SkillNode) => {
    setEditingNode(node);
    setFormData({
      name: node.name,
      description: node.description,
      level: node.level,
    });
  };

  const handleDelete = async (id: string) => {
    if (confirm('确定要删除这个技能节点吗？')) {
      await deleteSkillNode(id);
      if (selectedNode?.id === id) setSelectedNode(null);
    }
  };

  // 绘制技能树
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

    const nodeWidth = 120;
    const nodeHeight = 60;
    const horizontalGap = 40;
    const verticalGap = 80;

    // 按层级分组
    const levels = [...new Set(skills.map((s) => s.level))].sort((a, b) => a - b);
    const nodesByLevel: SkillNode[][] = levels.map(() => []);

    skills.forEach((skill) => {
      const levelIndex = levels.indexOf(skill.level);
      if (levelIndex !== -1) {
        nodesByLevel[levelIndex].push(skill);
      }
    });

    // 绘制连接线
    skills.forEach((skill) => {
      if (skill.parentId) {
        const parent = skills.find((s) => s.id === skill.parentId);
        if (!parent) return;

        const parentLevelIndex = levels.indexOf(parent.level);
        const childLevelIndex = levels.indexOf(skill.level);
        const parentIndex = nodesByLevel[parentLevelIndex].indexOf(parent);
        const childIndex = nodesByLevel[childLevelIndex].indexOf(skill);

        const parentX =
          (rect.width / (nodesByLevel[parentLevelIndex].length + 1)) * (parentIndex + 1);
        const parentY = 60 + parentLevelIndex * (nodeHeight + verticalGap) + nodeHeight / 2;
        const childX =
          (rect.width / (nodesByLevel[childLevelIndex].length + 1)) * (childIndex + 1);
        const childY = 60 + childLevelIndex * (nodeHeight + verticalGap);

        ctx.beginPath();
        ctx.moveTo(parentX, parentY);
        ctx.lineTo(childX, childY);
        ctx.strokeStyle = '#d4a574';
        ctx.lineWidth = 2;
        ctx.stroke();
      }
    });

    // 绘制节点
    nodesByLevel.forEach((levelNodes, levelIndex) => {
      levelNodes.forEach((node, index) => {
        const x = (rect.width / (levelNodes.length + 1)) * (index + 1) - nodeWidth / 2;
        const y = 60 + levelIndex * (nodeHeight + verticalGap);

        // 选中高亮
        if (selectedNode?.id === node.id) {
          ctx.fillStyle = '#d4a574';
          ctx.fillRect(x - 4, y - 4, nodeWidth + 8, nodeHeight + 8);
        }

        // 节点背景
        ctx.fillStyle = '#21262d';
        ctx.fillRect(x, y, nodeWidth, nodeHeight);
        ctx.strokeStyle = node.level === 0 ? '#d4a574' : '#58a6ff';
        ctx.lineWidth = 2;
        ctx.strokeRect(x, y, nodeWidth, nodeHeight);

        // 文字
        ctx.fillStyle = '#e6edf3';
        ctx.font = 'bold 14px Microsoft YaHei';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(node.name.substring(0, 6), x + nodeWidth / 2, y + nodeHeight / 2 - 8);

        ctx.font = '10px Microsoft YaHei';
        ctx.fillStyle = '#8b949e';
        ctx.fillText(
          `Lv.${node.level}`,
          x + nodeWidth / 2,
          y + nodeHeight / 2 + 14
        );

        // 点击区域
        const canvasNode = { x, y, width: nodeWidth, height: nodeHeight, node };
        canvas.onclick = (e) => {
          const rect = canvas.getBoundingClientRect();
          const clickX = e.clientX - rect.left;
          const clickY = e.clientY - rect.top;
          if (
            clickX >= canvasNode.x &&
            clickX <= canvasNode.x + canvasNode.width &&
            clickY >= canvasNode.y &&
            clickY <= canvasNode.y + canvasNode.height
          ) {
            setSelectedNode(canvasNode.node);
          }
        };
      });
    });
  }, [skills, selectedNode]);

  return (
    <div className="h-full flex p-6 gap-6">
      {/* 技能列表 */}
      <div className="w-64 flex flex-col">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-[var(--color-text-primary)]">技能树</h2>
          <button onClick={() => setShowForm(true)} className="btn-primary p-2">
            <Plus size={16} />
          </button>
        </div>

        {(showForm || editingNode) && (
          <div className="card mb-4 p-3">
            <input
              type="text"
              value={formData.name || ''}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder="技能名称"
              className="input-base mb-2"
            />
            <textarea
              value={formData.description || ''}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="技能描述"
              className="input-base mb-2 min-h-[60px]"
            />
            <div className="mb-2">
              <label className="text-xs text-[var(--color-text-muted)] mb-1 block">等级</label>
              <input
                type="number"
                min="0"
                max="10"
                value={formData.level || 0}
                onChange={(e) => setFormData({ ...formData, level: parseInt(e.target.value) || 0 })}
                className="input-base"
              />
            </div>
            <div className="flex gap-2">
              <button onClick={editingNode ? handleUpdate : handleAdd} className="btn-primary flex-1 text-sm">
                {editingNode ? '保存' : '添加'}
              </button>
              <button
                onClick={() => {
                  setShowForm(false);
                  setEditingNode(null);
                  setFormData({ name: '', description: '', level: 0 });
                }}
                className="btn-secondary flex-1 text-sm"
              >
                取消
              </button>
            </div>
          </div>
        )}

        <div className="flex-1 overflow-auto space-y-2">
          {skills.map((node) => (
            <div
              key={node.id}
              onClick={() => setSelectedNode(node)}
              className={clsx(
                'p-3 rounded-lg cursor-pointer transition-colors',
                selectedNode?.id === node.id
                  ? 'bg-[var(--color-bg-tertiary)] border border-[var(--color-accent-gold)]'
                  : 'bg-[var(--color-bg-secondary)] border border-transparent hover:border-[var(--color-border)]'
              )}
            >
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="font-medium text-[var(--color-text-primary)]">{node.name}</h3>
                  <span className="text-xs text-[var(--color-accent-cyan)]">等级 {node.level}</span>
                </div>
                <div className="flex gap-1">
                  <button onClick={() => handleEdit(node)} className="p-1 text-[var(--color-text-muted)]">
                    <GitBranch size={14} />
                  </button>
                  <button
                    onClick={() => handleDelete(node.id)}
                    className="p-1 text-[var(--color-text-muted)] hover:text-[var(--color-error)]"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            </div>
          ))}

          {skills.length === 0 && (
            <div className="text-center py-8 text-[var(--color-text-muted)]">
              <GitBranch size={32} className="mx-auto mb-2" />
              <p className="text-sm">暂无技能节点</p>
            </div>
          )}
        </div>
      </div>

      {/* 技能树画布 */}
      <div className="flex-1 card flex flex-col">
        <h3 className="font-medium text-[var(--color-text-primary)] mb-3">技能树可视化</h3>
        <canvas ref={canvasRef} className="flex-1 bg-[var(--color-bg-primary)] rounded-lg" />
      </div>

      {/* 选中节点详情 */}
      {selectedNode && (
        <div className="w-64 card">
          <h3 className="font-medium text-[var(--color-text-primary)] mb-3">{selectedNode.name}</h3>
          <p className="text-sm text-[var(--color-text-secondary)]">
            {selectedNode.description || '暂无描述'}
          </p>
          <div className="mt-4 text-xs text-[var(--color-text-muted)]">
            <p>等级: {selectedNode.level}</p>
          </div>
        </div>
      )}
    </div>
  );
};
