import React, { useState } from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { Plus, Trash2, Edit2, Clock, User } from 'lucide-react';
import { clsx } from 'clsx';
import type { TimelineEvent } from '@/types';

export const TimelineView: React.FC = () => {
  const { timeline, addTimelineEvent, updateTimelineEvent, deleteTimelineEvent } = useProjectStore();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<Partial<TimelineEvent>>({
    title: '',
    description: '',
    timestamp: '',
    type: 'main',
  });

  const handleAdd = async () => {
    if (!formData.title || !formData.timestamp) return;
    await addTimelineEvent(formData as Omit<TimelineEvent, 'id'>);
    setShowForm(false);
    setFormData({ title: '', description: '', timestamp: '', type: 'main' });
  };

  const handleUpdate = async () => {
    if (!editingId) return;
    const event = timeline.find((e) => e.id === editingId);
    if (event) {
      await updateTimelineEvent({ ...event, ...formData });
    }
    setEditingId(null);
    setFormData({ title: '', description: '', timestamp: '', type: 'main' });
  };

  const handleEdit = (event: TimelineEvent) => {
    setEditingId(event.id);
    setFormData({
      title: event.title,
      description: event.description,
      timestamp: event.timestamp,
      type: event.type,
    });
  };

  const handleDelete = async (id: string) => {
    if (confirm('确定要删除这个事件吗？')) {
      await deleteTimelineEvent(id);
    }
  };

  const sortedTimeline = [...timeline].sort((a, b) => a.timestamp.localeCompare(b.timestamp));

  return (
    <div className="h-full flex flex-col p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-[var(--color-text-primary)]">时间轴</h2>
        <button onClick={() => setShowForm(true)} className="btn-primary flex items-center gap-2">
          <Plus size={16} />
          添加事件
        </button>
      </div>

      {/* 添加/编辑表单 */}
      {(showForm || editingId) && (
        <div className="card mb-6 p-4">
          <h3 className="font-medium text-[var(--color-text-primary)] mb-3">
            {editingId ? '编辑事件' : '添加新事件'}
          </h3>
          <div className="grid gap-3">
            <input
              type="text"
              value={formData.title || ''}
              onChange={(e) => setFormData({ ...formData, title: e.target.value })}
              placeholder="事件标题"
              className="input-base"
            />
            <input
              type="text"
              value={formData.timestamp || ''}
              onChange={(e) => setFormData({ ...formData, timestamp: e.target.value })}
              placeholder="时间点 (如: 第1年3月)"
              className="input-base"
            />
            <textarea
              value={formData.description || ''}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="事件描述"
              className="input-base min-h-[80px]"
            />
            <div className="flex gap-2">
              {(['main', 'branch', 'custom'] as const).map((type) => (
                <button
                  key={type}
                  onClick={() => setFormData({ ...formData, type })}
                  className={clsx(
                    'px-3 py-1 rounded-full text-sm transition-colors',
                    formData.type === type
                      ? 'bg-[var(--color-accent-gold)] text-[var(--color-bg-primary)]'
                      : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)]'
                  )}
                >
                  {type === 'main' ? '主线' : type === 'branch' ? '支线' : '自定义'}
                </button>
              ))}
            </div>
            <div className="flex gap-2">
              <button
                onClick={editingId ? handleUpdate : handleAdd}
                className="btn-primary flex-1"
              >
                {editingId ? '保存' : '添加'}
              </button>
              <button
                onClick={() => {
                  setShowForm(false);
                  setEditingId(null);
                  setFormData({ title: '', description: '', timestamp: '', type: 'main' });
                }}
                className="btn-secondary flex-1"
              >
                取消
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 时间轴列表 */}
      <div className="flex-1 overflow-auto">
        {sortedTimeline.length === 0 ? (
          <div className="text-center py-12 text-[var(--color-text-muted)]">
            <Clock size={48} className="mx-auto mb-4" />
            <p>暂无时间轴事件</p>
            <button onClick={() => setShowForm(true)} className="btn-secondary mt-4">
              添加第一个事件
            </button>
          </div>
        ) : (
          <div className="relative">
            {/* 时间线 */}
            <div className="absolute left-6 top-0 bottom-0 w-px bg-[var(--color-border)]" />
            <div className="space-y-4">
              {sortedTimeline.map((event) => (
                <div key={event.id} className="relative flex gap-4">
                  <div
                    className={clsx(
                      'w-12 h-12 rounded-full flex items-center justify-center z-10',
                      event.type === 'main'
                        ? 'bg-[var(--color-accent-gold)]'
                        : event.type === 'branch'
                        ? 'bg-[var(--color-accent-cyan)]'
                        : 'bg-[var(--color-bg-elevated)]'
                    )}
                  >
                    <Clock size={20} className="text-[var(--color-bg-primary)]" />
                  </div>
                  <div className="flex-1 card">
                    <div className="flex items-start justify-between">
                      <div>
                        <span className="text-xs text-[var(--color-accent-cyan)] font-mono">
                          {event.timestamp}
                        </span>
                        <h4 className="font-medium text-[var(--color-text-primary)] mt-1">
                          {event.title}
                        </h4>
                        <p className="text-sm text-[var(--color-text-secondary)] mt-1">
                          {event.description}
                        </p>
                      </div>
                      <div className="flex gap-1">
                        <button
                          onClick={() => handleEdit(event)}
                          className="p-1 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)]"
                        >
                          <Edit2 size={14} />
                        </button>
                        <button
                          onClick={() => handleDelete(event.id)}
                          className="p-1 rounded hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] hover:text-[var(--color-error)]"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
