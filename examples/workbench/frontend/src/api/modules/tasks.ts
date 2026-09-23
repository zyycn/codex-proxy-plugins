import type { SavedTasks, SavedTasksReply } from '../types/workbench'
import request from '../request'
import { parseSavedTasks } from '../schemas/tasks'
import { nonNegativeInteger, record } from '../validation'

export function getSavedTasks(): Promise<SavedTasksReply> {
  return request({
    url: 'api/tasks',
    method: 'GET',
  }).then(parseSavedTasks)
}

export async function saveTasks(expectedVersion: number | null, value: SavedTasks): Promise<number> {
  const reply = await request({
    url: 'api/tasks',
    method: 'POST',
    data: { expectedVersion, value },
  })
  return nonNegativeInteger(record(reply, '保存结果').version, '状态版本')
}
