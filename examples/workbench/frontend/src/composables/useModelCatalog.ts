// @env browser
import type { Ref } from 'vue'
import type { WorkbenchSnapshot } from '../api'
import type { UiSelectOption } from '../types'
import { computed, readonly, shallowRef, watch } from 'vue'
import { listModels } from '../api'

export function useModelCatalog(snapshot: Ref<WorkbenchSnapshot | undefined>) {
  const clientKeyId = shallowRef('')
  const modelId = shallowRef('')
  const models = shallowRef<string[]>([])
  const loading = shallowRef(false)
  const error = shallowRef('')
  const keyOptions = computed<UiSelectOption[]>(() => (snapshot.value?.keys ?? []).map(key => ({
    label: key.name,
    value: key.id,
    disabled: !key.enabled,
    description: key.enabled ? undefined : '已停用',
  })))
  const modelOptions = computed(() => models.value.map(model => ({ label: model, value: model })))

  watch(() => snapshot.value?.keys, (keys) => {
    const enabled = keys?.filter(key => key.enabled) ?? []
    if (!enabled.some(key => key.id === clientKeyId.value))
      clientKeyId.value = enabled[0]?.id ?? ''
  }, { immediate: true })

  watch(clientKeyId, async (key, _previous, onCleanup) => {
    let active = true
    onCleanup(() => {
      active = false
    })
    models.value = []
    error.value = ''
    loading.value = Boolean(key)
    if (!key) {
      modelId.value = ''
      return
    }
    try {
      const next = await listModels(key)
      if (!active)
        return
      models.value = next
      if (!next.includes(modelId.value))
        modelId.value = next[0] ?? ''
    }
    catch (cause) {
      if (active) {
        modelId.value = ''
        error.value = cause instanceof Error ? cause.message : '无法载入模型，请切换 Key 后重试'
      }
    }
    finally {
      if (active)
        loading.value = false
    }
  }, { immediate: true })

  return { clientKeyId, modelId, keyOptions, modelOptions, loading: readonly(loading), error: readonly(error) }
}
