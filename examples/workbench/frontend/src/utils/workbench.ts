import type { SavedTask, TaskKind } from '../api'
import { defaultInstructions, taskLabels } from '../constants/workbench'

export function realModelIds(models: string[], demoModels: string[]): string[] {
  const demos = new Set(demoModels)
  return models.filter(model => !demos.has(model))
}

export function instructionFor(task: TaskKind, custom: string): string {
  return custom.trim() || defaultInstructions[task]
}

export function taskLabel(task: TaskKind): string {
  return taskLabels[task]
}

export function buildInitialRequest(
  model: string,
  task: TaskKind,
  source: string,
  customInstruction: string,
): Record<string, unknown> {
  const instruction = instructionFor(task, customInstruction)
  return {
    model,
    stream: true,
    store: false,
    instructions: '你是文本编辑助手，只完成用户指定的文本任务，不编造来源之外的事实',
    input: `${instruction}\n\n待处理文本：\n${source}`,
  }
}

export function buildContinuationRequest(
  model: string,
  task: TaskKind,
  source: string,
  customInstruction: string,
  result: string,
  adjustment: string,
): Record<string, unknown> {
  return {
    model,
    stream: true,
    store: false,
    instructions: '你是文本编辑助手，只完成用户指定的文本任务，不编造来源之外的事实',
    input: [
      { role: 'user', content: `${instructionFor(task, customInstruction)}\n\n待处理文本：\n${source}` },
      { role: 'assistant', content: result },
      { role: 'user', content: adjustment },
    ],
  }
}

export function savedTaskTitle(source: string, task: TaskKind): string {
  const firstLine = source.trim().split(/\r?\n/, 1)[0]?.replace(/\s+/g, ' ') ?? ''
  const excerpt = firstLine.length > 28 ? `${firstLine.slice(0, 28)}…` : firstLine
  return excerpt || taskLabels[task]
}

export function upsertSavedTask(tasks: SavedTask[], task: SavedTask): SavedTask[] {
  return [task, ...tasks.filter(item => item.id !== task.id)]
    .sort((left, right) => right.updatedAt - left.updatedAt)
    .slice(0, 8)
}

export function formatDateTime(value: number): string {
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(new Date(value))
}

export function formatBytes(value: number): string {
  if (value < 1024)
    return `${value} B`
  return `${(value / 1024).toFixed(value < 10 * 1024 ? 1 : 0)} KiB`
}
