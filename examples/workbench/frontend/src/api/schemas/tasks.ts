import type { SavedTask, SavedTasks, SavedTasksReply } from '../types/workbench'
import { sourceKinds, taskKinds } from '@/constants/workbench'
import { nonNegativeInteger, nullableString, record, string } from '../validation'

function parseTask(value: unknown): SavedTask {
  const item = record(value, '保存记录')
  if (typeof item.sourceKind !== 'string' || !sourceKinds.has(item.sourceKind))
    throw new Error('来源类型格式无效')
  if (typeof item.task !== 'string' || !taskKinds.has(item.task))
    throw new Error('任务类型格式无效')
  return {
    id: string(item.id, '记录 ID'),
    title: string(item.title, '记录标题'),
    sourceKind: item.sourceKind as SavedTask['sourceKind'],
    source: string(item.source, '原文'),
    sourceUrl: nullableString(item.sourceUrl, '网页地址'),
    task: item.task as SavedTask['task'],
    instruction: string(item.instruction, '任务要求'),
    result: string(item.result, '结果'),
    modelId: string(item.modelId, '模型 ID'),
    clientKeyId: string(item.clientKeyId, 'Key ID'),
    updatedAt: nonNegativeInteger(item.updatedAt, '更新时间'),
  }
}

function parseTasks(value: unknown): SavedTasks {
  const item = record(value, '历史记录')
  if (!Array.isArray(item.entries) || item.entries.length > 8)
    throw new Error('历史记录格式无效')
  return {
    selectedId: nullableString(item.selectedId, '当前记录'),
    entries: item.entries.map(parseTask),
  }
}

export function parseSavedTasks(input: unknown): SavedTasksReply {
  const value = record(input, '历史记录')
  const version = value.version === null ? null : nonNegativeInteger(value.version, '状态版本')
  return { version, value: value.value === null ? null : parseTasks(value.value) }
}
