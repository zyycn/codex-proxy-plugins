// @env browser
import type { WorkbenchSnapshot } from '../api'
import { onMounted, readonly, shallowReadonly, shallowRef } from 'vue'
import { getSnapshot } from '../api'

export function useWorkbenchData() {
  const snapshot = shallowRef<WorkbenchSnapshot>()
  const loading = shallowRef(false)
  const error = shallowRef('')

  async function refresh() {
    if (loading.value)
      return
    loading.value = true
    error.value = ''
    try {
      snapshot.value = await getSnapshot()
    }
    catch (cause) {
      error.value = cause instanceof Error ? cause.message : '读取工作台失败'
    }
    finally {
      loading.value = false
    }
  }

  onMounted(refresh)

  return {
    snapshot: shallowReadonly(snapshot),
    loading: readonly(loading),
    error: readonly(error),
    refresh,
  }
}
