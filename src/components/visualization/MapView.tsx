import React, { useRef, useState, useEffect } from 'react';
import { useProjectStore } from '@/stores/projectStore';
import { Plus, Trash2, MapPin, Crosshair } from 'lucide-react';
import type { MapMarker } from '@/types';

export const MapView: React.FC = () => {
  const { mapData, updateMapData } = useProjectStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isDrawing, setIsDrawing] = useState(false);
  const [tool, setTool] = useState<'select' | 'marker' | 'draw'>('select');
  const [showForm, setShowForm] = useState(false);
  const [editingMarker, setEditingMarker] = useState<MapMarker | null>(null);
  const [formData, setFormData] = useState<Partial<MapMarker>>({
    name: '',
    type: 'city',
    x: 0,
    y: 0,
  });

  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (tool === 'select') return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (tool === 'marker') {
      setFormData({ ...formData, x, y, name: '', type: 'city' });
      setShowForm(true);
    }
  };

  const handleAddMarker = () => {
    if (!formData.name || formData.x === undefined || formData.y === undefined) return;

    const newMarker: MapMarker = {
      id: editingMarker?.id || `marker-${Date.now()}`,
      name: formData.name,
      type: formData.type as MapMarker['type'],
      x: formData.x,
      y: formData.y,
      description: formData.description,
      connections: formData.connections,
    };

    let newMarkers: MapMarker[];
    if (editingMarker) {
      newMarkers = mapData.markers.map((m) => (m.id === editingMarker.id ? newMarker : m));
    } else {
      newMarkers = [...mapData.markers, newMarker];
    }

    updateMapData({ ...mapData, markers: newMarkers });
    setShowForm(false);
    setEditingMarker(null);
    setFormData({ name: '', type: 'city', x: 0, y: 0 });
  };

  const handleEditMarker = (marker: MapMarker) => {
    setEditingMarker(marker);
    setFormData(marker);
    setShowForm(true);
  };

  const handleDeleteMarker = (id: string) => {
    if (confirm('确定要删除这个标记吗？')) {
      updateMapData({
        ...mapData,
        markers: mapData.markers.filter((m) => m.id !== id),
      });
    }
  };

  // 绘制地图
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * window.devicePixelRatio;
    canvas.height = rect.height * window.devicePixelRatio;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);

    // 背景
    ctx.fillStyle = '#0d1117';
    ctx.fillRect(0, 0, rect.width, rect.height);

    // 网格
    ctx.strokeStyle = '#21262d';
    ctx.lineWidth = 1;
    for (let x = 0; x < rect.width; x += 40) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, rect.height);
      ctx.stroke();
    }
    for (let y = 0; y < rect.height; y += 40) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(rect.width, y);
      ctx.stroke();
    }

    // 绘制连接线
    mapData.markers.forEach((marker) => {
      if (marker.connections) {
        marker.connections.forEach((connId) => {
          const target = mapData.markers.find((m) => m.id === connId);
          if (!target) return;

          ctx.beginPath();
          ctx.moveTo(marker.x, marker.y);
          ctx.lineTo(target.x, target.y);
          ctx.strokeStyle = '#58a6ff';
          ctx.lineWidth = 2;
          ctx.setLineDash([5, 5]);
          ctx.stroke();
          ctx.setLineDash([]);
        });
      }
    });

    // 绘制标记
    mapData.markers.forEach((marker) => {
      const colors: Record<MapMarker['type'], string> = {
        city: '#d4a574',
        region: '#58a6ff',
        landmark: '#3fb950',
        route: '#8b949e',
      };

      ctx.beginPath();
      ctx.arc(marker.x, marker.y, 12, 0, Math.PI * 2);
      ctx.fillStyle = colors[marker.type];
      ctx.fill();
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 2;
      ctx.stroke();

      // 标签
      ctx.fillStyle = '#e6edf3';
      ctx.font = '12px Microsoft YaHei';
      ctx.textAlign = 'center';
      ctx.fillText(marker.name, marker.x, marker.y + 28);
    });
  }, [mapData]);

  return (
    <div className="h-full flex flex-col p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-[var(--color-text-primary)]">地图设计</h2>
        <div className="flex gap-2">
          <button
            onClick={() => setTool('select')}
            className={`px-3 py-1 rounded text-sm ${
              tool === 'select' ? 'bg-[var(--color-accent-gold)] text-[var(--color-bg-primary)]' : 'btn-secondary'
            }`}
          >
            选择
          </button>
          <button
            onClick={() => setTool('marker')}
            className={`px-3 py-1 rounded text-sm ${
              tool === 'marker' ? 'bg-[var(--color-accent-gold)] text-[var(--color-bg-primary)]' : 'btn-secondary'
            }`}
          >
            <MapPin size={14} className="inline mr-1" />
            添加标记
          </button>
        </div>
      </div>

      <div className="flex-1 flex gap-4">
        {/* 地图画布 */}
        <div className="flex-1 card p-0 overflow-hidden">
          <canvas
            ref={canvasRef}
            className="w-full h-full cursor-crosshair"
            onClick={handleCanvasClick}
          />
        </div>

        {/* 标记列表 */}
        <div className="w-64 flex flex-col">
          <h3 className="font-medium text-[var(--color-text-primary)] mb-3">地点列表</h3>
          <div className="flex-1 overflow-auto space-y-2">
            {mapData.markers.map((marker) => {
              const typeLabels: Record<MapMarker['type'], string> = {
                city: '城市',
                region: '区域',
                landmark: '地标',
                route: '路线',
              };
              return (
                <div
                  key={marker.id}
                  className="p-3 rounded-lg bg-[var(--color-bg-secondary)] border border-[var(--color-border)]"
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="font-medium text-[var(--color-text-primary)]">{marker.name}</span>
                    <span className="text-xs text-[var(--color-accent-cyan)]">{typeLabels[marker.type]}</span>
                  </div>
                  <p className="text-xs text-[var(--color-text-muted)]">
                    位置: ({Math.round(marker.x)}, {Math.round(marker.y)})
                  </p>
                  <div className="flex gap-2 mt-2">
                    <button
                      onClick={() => handleEditMarker(marker)}
                      className="text-xs text-[var(--color-accent-cyan)] hover:underline"
                    >
                      编辑
                    </button>
                    <button
                      onClick={() => handleDeleteMarker(marker.id)}
                      className="text-xs text-[var(--color-error)] hover:underline"
                    >
                      删除
                    </button>
                  </div>
                </div>
              );
            })}

            {mapData.markers.length === 0 && (
              <div className="text-center py-8 text-[var(--color-text-muted)]">
                <MapPin size={32} className="mx-auto mb-2" />
                <p className="text-sm">暂无地点</p>
                <p className="text-xs mt-1">点击"添加标记"后在地图上点击添加</p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 添加/编辑弹窗 */}
      {showForm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card w-full max-w-sm p-6">
            <h3 className="text-lg font-semibold text-[var(--color-text-primary)] mb-4">
              {editingMarker ? '编辑地点' : '添加地点'}
            </h3>
            <div className="space-y-3">
              <input
                type="text"
                value={formData.name || ''}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="地点名称"
                className="input-base"
              />
              <select
                value={formData.type || 'city'}
                onChange={(e) => setFormData({ ...formData, type: e.target.value as MapMarker['type'] })}
                className="input-base"
              >
                <option value="city">城市</option>
                <option value="region">区域</option>
                <option value="landmark">地标</option>
                <option value="route">路线</option>
              </select>
              <textarea
                value={formData.description || ''}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                placeholder="地点描述（可选）"
                className="input-base min-h-[60px]"
              />
              <div className="flex gap-3">
                <button onClick={handleAddMarker} className="btn-primary flex-1">
                  {editingMarker ? '保存' : '添加'}
                </button>
                <button
                  onClick={() => {
                    setShowForm(false);
                    setEditingMarker(null);
                  }}
                  className="btn-secondary flex-1"
                >
                  取消
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
