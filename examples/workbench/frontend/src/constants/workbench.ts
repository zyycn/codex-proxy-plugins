import type { TaskKind } from '../api'
import type { GenerationPhase, UiSegmentedOption } from '../types'
import { Blocks, BookOpenText } from '@lucide/vue'

export const taskOptions: UiSegmentedOption[] = [
  { label: '摘要', value: 'summarize' },
  { label: '翻译', value: 'translate' },
  { label: '改写', value: 'rewrite' },
]

export const defaultInstructions: Record<TaskKind, string> = {
  summarize: '提炼核心观点和关键事实，使用清楚的短段落，不添加原文没有的信息',
  translate: '翻译为简体中文，保留标题、段落结构、数字和专有名词',
  rewrite: '改写得清晰、自然、专业，保持事实和原意不变',
}

export const taskLabels: Record<TaskKind, string> = {
  summarize: '生成摘要',
  translate: '翻译文本',
  rewrite: '改写文本',
}

export const exampleStatuses = new Set(['not_run', 'passed', 'failed', 'pending'])
export const taskKinds = new Set(['summarize', 'translate', 'rewrite'])
export const sourceKinds = new Set(['text', 'url'])

export const workbenchViews: UiSegmentedOption[] = [
  { label: '文本工作台', value: 'text', icon: BookOpenText },
  { label: '基础示例', value: 'examples', icon: Blocks },
]

export const sourceKindsOptions: UiSegmentedOption[] = [
  { label: '粘贴文本', value: 'text' },
  { label: '读取链接', value: 'url' },
]

export const generationLabels: Record<GenerationPhase, string> = {
  idle: '等待输入',
  requesting: '正在连接',
  streaming: '正在生成',
  done: '已完成',
  stopped: '已停止',
  error: '生成失败',
}
