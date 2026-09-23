// @env browser
import type { SavedTask, SavedTasks } from '../api'
import { computed, readonly, shallowReadonly, shallowRef } from 'vue'
import { getSavedTasks, saveTasks } from '../api'
import { upsertSavedTask } from '../utils/workbench'

export function useTaskHistory() {
  const tasks = shallowRef<SavedTasks>({ selectedId: null, entries: [] })
  const version = shallowRef<number | null>(null)
  const loading = shallowRef(false)
  const saving = shallowRef(false)
  const error = shallowRef('')
  const notice = shallowRef('')
  const options = computed(() => [
    { label: '新任务', value: '' },
    ...tasks.value.entries.map(item => ({
      label: item.title,
      value: item.id,
      description: new Intl.DateTimeFormat('zh-CN', {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
      }).format(new Date(item.updatedAt)),
    })),
  ])

  function clearFeedback() {
    error.value = ''
    notice.value = ''
  }

  async function load() {
    if (loading.value || saving.value)
      return
    loading.value = true
    error.value = ''
    notice.value = ''
    try {
      const reply = await getSavedTasks()
      version.value = reply.version
      tasks.value = reply.value ?? { selectedId: null, entries: [] }
      return tasks.value
    }
    catch (cause) {
      error.value = cause instanceof Error ? cause.message : '无法载入历史记录，请重试'
    }
    finally {
      loading.value = false
    }
  }

  async function save(entry: SavedTask) {
    if (saving.value || loading.value)
      return false
    saving.value = true
    error.value = ''
    notice.value = ''
    const next = { selectedId: entry.id, entries: upsertSavedTask(tasks.value.entries, entry) }
    try {
      version.value = await saveTasks(version.value, next)
      tasks.value = next
      notice.value = '已保存到历史记录'
      return true
    }
    catch (cause) {
      error.value = cause instanceof Error ? cause.message : '保存失败，请刷新历史记录后重试'
      return false
    }
    finally {
      saving.value = false
    }
  }

  return {
    tasks: shallowReadonly(tasks),
    options,
    loading: readonly(loading),
    saving: readonly(saving),
    error: readonly(error),
    notice: readonly(notice),
    load,
    save,
    clearFeedback,
  }
}
