import type {
  ChatMessage,
  GenerateOptions,
  PlotBranch,
  ValidationResult,
  DeepSeekResponse,
} from '@/types';

const API_BASE = 'https://api.deepseek.com/v1';

class DeepSeekService {
  private apiKey: string = '';

  setApiKey(key: string) {
    this.apiKey = key;
  }

  private async request<T>(
    endpoint: string,
    body: Record<string, unknown>,
    options?: GenerateOptions
  ): Promise<T> {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${this.apiKey}`,
      },
      body: JSON.stringify({
        model: options?.model || 'deepseek-chat',
        temperature: options?.temperature ?? 0.7,
        max_tokens: options?.maxTokens ?? 2048,
        ...body,
      }),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: { message: 'Request failed' } }));
      throw new Error(error.error?.message || `API request failed: ${response.status}`);
    }

    return response.json();
  }

  async chat(messages: ChatMessage[], options?: GenerateOptions): Promise<string> {
    const data = await this.request<DeepSeekResponse>(
      '/chat/completions',
      { messages },
      options
    );
    return data.choices[0]?.message?.content || '';
  }

  async generateContinuation(
    currentText: string,
    context?: string,
    options?: GenerateOptions
  ): Promise<string> {
    const systemPrompt = `你是小说续写助手。根据提供的上下文，续写故事。
要求：
- 保持原有文风和角色性格
- 推进剧情发展
- 避免重复已有内容
- 续写长度适中，500-1000字`;

    const userPrompt = context
      ? `上下文：\n${context}\n\n当前文本：\n${currentText}\n\n续写内容：`
      : `当前文本：\n${currentText}\n\n续写内容：`;

    return this.chat(
      [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: userPrompt },
      ],
      { temperature: 0.8, maxTokens: 2000, ...options }
    );
  }

  async generateSummary(content: string, options?: GenerateOptions): Promise<string> {
    const systemPrompt = `你是小说章节摘要助手。分析以下章节内容，生成简洁准确的摘要。
要求：
- 100-200字
- 包含主要情节和关键人物
- 不包含具体细节，仅提炼核心内容`;

    return this.chat(
      [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: `章节内容：\n${content}` },
      ],
      { temperature: 0.5, maxTokens: 500, ...options }
    );
  }

  async generatePlotTree(
    outline: string,
    branches: number = 3,
    options?: GenerateOptions
  ): Promise<PlotBranch[]> {
    const systemPrompt = `你是剧情设计师。基于提供的大纲，生成分支剧情树。
要求：
- 生成 ${branches} 个主要分支
- 每个分支包含剧情走向描述
- 标注关键转折点
- 列出可能的后果

输出格式为 JSON：
{
  "branches": [
    {
      "id": "branch-1",
      "description": "剧情描述",
      "consequences": ["后果1", "后果2"]
    }
  ]
}`;

    const response = await this.chat(
      [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: `已有大纲：\n${outline}` },
      ],
      { temperature: 0.8, maxTokens: 3000, ...options }
    );

    // 尝试解析 JSON
    try {
      const jsonMatch = response.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        const parsed = JSON.parse(jsonMatch[0]);
        return parsed.branches || [];
      }
    } catch {
      // 如果解析失败，返回文本
      return [{ id: '1', description: response, consequences: [] }];
    }

    return [];
  }

  async generateNames(
    type: 'character' | 'place' | 'faction',
    style: string = 'default',
    count: number = 10,
    options?: GenerateOptions
  ): Promise<string[]> {
    const typeMap = {
      character: '人物姓名',
      place: '地名',
      faction: '势力/组织名称',
    };

    const styleMap: Record<string, string> = {
      default: '古风/东方奇幻风格',
      western: '西方奇幻风格',
      scifi: '科幻风格',
      modern: '现代风格',
      xianxia: '仙侠风格',
    };

    const systemPrompt = `你是命名助手。生成 ${count} 个${styleMap[style] || style}的${typeMap[type]}。
要求：
- 每个名称独特有意义
- 符合指定风格
- 直接输出列表，每行一个名称，不要编号`;

    const response = await this.chat(
      [{ role: 'system', content: systemPrompt }],
      { temperature: 0.9, maxTokens: 1000, ...options }
    );

    return response
      .split('\n')
      .map((name) => name.trim())
      .filter((name) => name.length > 0 && name.length < 20);
  }

  async validateTimeline(
    events: { timestamp: string; description: string }[],
    worldContext: string,
    options?: GenerateOptions
  ): Promise<ValidationResult[]> {
    const systemPrompt = `你是时间线校验助手。检查以下事件列表是否存在矛盾。
检查项：
1. 因果矛盾（原因发生在结果之前）
2. 时间线冲突（同一人物出现在不同时间点）
3. 季节/天气矛盾
4. 逻辑矛盾

输出格式为 JSON：
{
  "issues": [
    {
      "type": "error" | "warning",
      "message": "问题描述",
      "location": "相关事件位置",
      "suggestion": "建议修改方案"
    }
  ]
}`;

    const eventsText = events
      .map((e, i) => `${i + 1}. [${e.timestamp}] ${e.description}`)
      .join('\n');

    const response = await this.chat(
      [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: `世界观背景：\n${worldContext}\n\n时间线事件：\n${eventsText}` },
      ],
      { temperature: 0.3, maxTokens: 2000, ...options }
    );

    try {
      const jsonMatch = response.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        const parsed = JSON.parse(jsonMatch[0]);
        return parsed.issues || [];
      }
    } catch {
      return [];
    }

    return [];
  }

  async validateWorldConsistency(
    newContent: string,
    existingContent: string[],
    options?: GenerateOptions
  ): Promise<ValidationResult[]> {
    const systemPrompt = `你是世界观一致性校验助手。检查新内容是否与已有世界观设定存在矛盾。
检查项：
1. 角色设定矛盾
2. 世界观规则矛盾
3. 地理/时间矛盾
4. 能力/技能体系矛盾

输出格式为 JSON：
{
  "issues": [
    {
      "type": "error" | "warning",
      "message": "问题描述",
      "location": "矛盾位置",
      "suggestion": "建议"
    }
  ]
}`;

    const existingText = existingContent.join('\n---\n');

    const response = await this.chat(
      [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: `已有内容：\n${existingText}\n\n新内容：\n${newContent}` },
      ],
      { temperature: 0.3, maxTokens: 2000, ...options }
    );

    try {
      const jsonMatch = response.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        const parsed = JSON.parse(jsonMatch[0]);
        return parsed.issues || [];
      }
    } catch {
      return [];
    }

    return [];
  }
}

export const deepseekService = new DeepSeekService();
