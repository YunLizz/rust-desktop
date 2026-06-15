import React, { useState } from 'react';
import { useAIStore } from '@/stores/aiStore';
import { Sparkles, Copy, RefreshCw, User, MapPin, Shield } from 'lucide-react';
import { clsx } from 'clsx';

type NameType = 'character' | 'place' | 'faction';

const TYPE_CONFIG: Record<NameType, { label: string; icon: React.ReactNode }> = {
  character: { label: '人物姓名', icon: <User size={16} /> },
  place: { label: '地名', icon: <MapPin size={16} /> },
  faction: { label: '势力名', icon: <Shield size={16} /> },
};

const STYLE_OPTIONS = [
  { value: 'default', label: '古风/东方' },
  { value: 'western', label: '西方奇幻' },
  { value: 'scifi', label: '科幻' },
  { value: 'modern', label: '现代' },
  { value: 'xianxia', label: '仙侠' },
];

export const NameGeneratorView: React.FC = () => {
  const { generateNames, isProcessing, isConfigured } = useAIStore();
  const [nameType, setNameType] = useState<NameType>('character');
  const [style, setStyle] = useState('default');
  const [count, setCount] = useState(10);
  const [names, setNames] = useState<string[]>([]);
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  const handleGenerate = async () => {
    if (!isConfigured()) {
      alert('请先配置 DeepSeek API Key');
      return;
    }
    try {
      const result = await generateNames(nameType, style, count);
      setNames(result);
    } catch (error) {
      alert((error as Error).message);
    }
  };

  const handleCopy = (name: string, index: number) => {
    navigator.clipboard.writeText(name);
    setCopiedIndex(index);
    setTimeout(() => setCopiedIndex(null), 1500);
  };

  const handleCopyAll = () => {
    navigator.clipboard.writeText(names.join('\n'));
  };

  return (
    <div className="h-full flex flex-col p-6">
      <div className="mb-6">
        <h2 className="text-xl font-semibold text-[var(--color-text-primary)] flex items-center gap-2">
          <Sparkles size={20} className="text-[var(--color-accent-gold)]" />
          起名机
        </h2>
        <p className="text-sm text-[var(--color-text-secondary)] mt-1">
          使用 AI 生成独特的人物姓名、地名和势力名称
        </p>
      </div>

      {/* 配置面板 */}
      <div className="card p-4 mb-6">
        <div className="grid gap-4 md:grid-cols-3">
          {/* 类型选择 */}
          <div>
            <label className="block text-sm font-medium text-[var(--color-text-primary)] mb-2">
              类型
            </label>
            <div className="flex gap-2">
              {(Object.keys(TYPE_CONFIG) as NameType[]).map((type) => (
                <button
                  key={type}
                  onClick={() => setNameType(type)}
                  className={clsx(
                    'flex-1 flex items-center justify-center gap-1 py-2 rounded-lg text-sm transition-colors',
                    nameType === type
                      ? 'bg-[var(--color-accent-gold)] text-[var(--color-bg-primary)]'
                      : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-elevated)]'
                  )}
                >
                  {TYPE_CONFIG[type].icon}
                </button>
              ))}
            </div>
          </div>

          {/* 风格选择 */}
          <div>
            <label className="block text-sm font-medium text-[var(--color-text-primary)] mb-2">
              风格
            </label>
            <select
              value={style}
              onChange={(e) => setStyle(e.target.value)}
              className="input-base"
            >
              {STYLE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>

          {/* 数量 */}
          <div>
            <label className="block text-sm font-medium text-[var(--color-text-primary)] mb-2">
              数量: {count}
            </label>
            <input
              type="range"
              min="5"
              max="20"
              value={count}
              onChange={(e) => setCount(parseInt(e.target.value))}
              className="w-full accent-[var(--color-accent-gold)]"
            />
          </div>
        </div>

        <button
          onClick={handleGenerate}
          disabled={isProcessing || !isConfigured()}
          className={clsx(
            'w-full mt-4 btn-primary flex items-center justify-center gap-2',
            isProcessing && 'opacity-50'
          )}
        >
          {isProcessing ? (
            <>
              <RefreshCw size={16} className="animate-spin" />
              生成中...
            </>
          ) : (
            <>
              <Sparkles size={16} />
              生成名称
            </>
          )}
        </button>
      </div>

      {/* 结果展示 */}
      <div className="flex-1 overflow-auto">
        {names.length > 0 ? (
          <>
            <div className="flex items-center justify-between mb-3">
              <h3 className="font-medium text-[var(--color-text-primary)]">
                生成结果 ({names.length})
              </h3>
              <button
                onClick={handleCopyAll}
                className="text-sm text-[var(--color-accent-cyan)] hover:underline flex items-center gap-1"
              >
                <Copy size={14} />
                复制全部
              </button>
            </div>
            <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
              {names.map((name, index) => (
                <div
                  key={index}
                  className="p-4 rounded-lg bg-[var(--color-bg-secondary)] border border-[var(--color-border)] hover:border-[var(--color-accent-gold)] transition-colors group"
                >
                  <div className="flex items-center justify-between">
                    <span className="text-[var(--color-text-primary)] font-medium">{name}</span>
                    <button
                      onClick={() => handleCopy(name, index)}
                      className="p-1 rounded opacity-0 group-hover:opacity-100 text-[var(--color-text-muted)] hover:text-[var(--color-accent-cyan)] transition-all"
                    >
                      <Copy size={14} />
                    </button>
                  </div>
                  {copiedIndex === index && (
                    <span className="text-xs text-[var(--color-success)]">已复制!</span>
                  )}
                </div>
              ))}
            </div>
          </>
        ) : (
          <div className="text-center py-12 text-[var(--color-text-muted)]">
            <Sparkles size={48} className="mx-auto mb-4 opacity-50" />
            <p>点击"生成名称"开始</p>
            <p className="text-sm mt-1">选择类型和风格，AI 将为你生成独特的名称</p>
          </div>
        )}
      </div>
    </div>
  );
};
