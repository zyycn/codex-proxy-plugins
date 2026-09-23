// @env browser
import type { Ref } from 'vue'
import type { WorkbenchSnapshot } from '../api'
import type { ExampleRun } from '../types'
import { computed, onBeforeUnmount, shallowRef, watch } from 'vue'
import { echo, listModels, modelResponses, prepareDemoAccounts } from '../api'
import { exampleGuides } from '../constants/examples'
import { consumeModelResponse } from '../utils/responses'

export function useExampleRunner(snapshot: Ref<WorkbenchSnapshot | undefined>, refresh: () => Promise<void>) {
  const selectedId = shallowRef('text-transform')
  const keyId = shallowRef('')
  const inputs = shallowRef<Record<string, string>>({})
  const runs = shallowRef<Record<string, ExampleRun>>({})
  const activeId = shallowRef('')
  let controller: AbortController | undefined
  let disposed = false

  const examples = computed(() => {
    const capabilities = new Set(snapshot.value?.examples.flatMap(item => item.capabilities) ?? [])
    return exampleGuides.filter(item => item.capabilities.every(capability => capabilities.has(capability)))
  })
  const selected = computed(() => examples.value.find(item => item.id === selectedId.value))
  const run = computed(() => runs.value[selectedId.value])
  const message = computed({
    get: () => inputs.value[selectedId.value] ?? (selected.value?.action === 'echo' ? '你好，插件' : 'Hello, plugin!'),
    set: (value: string) => { inputs.value = { ...inputs.value, [selectedId.value]: value } },
  })
  const keyOptions = computed(() => (snapshot.value?.keys ?? []).map(key => ({
    label: key.name,
    value: key.id,
    disabled: !key.enabled,
  })))
  const evidence = computed(() => {
    const current = run.value
    const capabilities = selected.value?.capabilities ?? []
    const entries = snapshot.value?.facts.entries ?? []
    if (selected.value?.group === 'integration')
      return capabilities.flatMap(capability => entries.find(item => item.capability === capability) ?? [])
    if (!current || (selected.value?.action !== 'echo' && !current.requestId))
      return []
    // 只展示本次请求，不把其他任务或上次运行的事实混入结果。
    return entries.filter(item => capabilities.includes(item.capability)
      && (current.requestId ? item.requestId === current.requestId : item.occurredAtMs >= current.startedAtMs))
  })

  watch(() => snapshot.value?.keys, (keys) => {
    const enabled = keys?.filter(key => key.enabled) ?? []
    if (!enabled.some(key => key.id === keyId.value))
      keyId.value = enabled[0]?.id ?? ''
  }, { immediate: true })

  function update(id: string, patch: Partial<ExampleRun>): void {
    if (!disposed && runs.value[id])
      runs.value = { ...runs.value, [id]: { ...runs.value[id], ...patch } }
  }

  async function execute(): Promise<void> {
    const guide = selected.value
    if (activeId.value || !guide || guide.action === 'external')
      return
    const id = guide.id
    const text = message.value.trim()
    const clientKeyId = keyId.value
    const abort = new AbortController()
    controller = abort
    activeId.value = id
    runs.value = { ...runs.value, [id]: {
      phase: 'running',
      input: text,
      output: '',
      error: '',
      accounts: [],
      requestId: null,
      model: null,
      inputTokens: null,
      outputTokens: null,
      totalTokens: null,
      chunks: 0,
      startedAtMs: Date.now(),
      durationMs: null,
    } }
    const startedAt = performance.now()
    try {
      if (guide.action === 'echo') {
        if (!text || new TextEncoder().encode(text).byteLength > 4096)
          throw new Error('请输入 1–4096 字节的文本')
        update(id, { output: await echo(text) })
      }
      else {
        if (guide.action !== 'accounts' && (!clientKeyId || !text))
          throw new Error('请选择可用 Key 并填写文本')
        const accounts = await prepareDemoAccounts()
        if (disposed || abort.signal.aborted)
          return
        update(id, { accounts })
        if (guide.action === 'accounts') {
          update(id, { output: '演示账号已就绪' })
        }
        else {
          const model = guide.action === 'request' ? 'demo-auto' : 'demo-echo'
          const models = await listModels(clientKeyId)
          if (disposed || abort.signal.aborted)
            return
          if (!models.includes(model))
            throw new Error('所选 Key 无法访问演示模型，请检查 Key 的模型和账号范围')
          const response = await modelResponses({
            clientKeyId,
            body: {
              model,
              input: text,
              stream: true,
              store: false,
              metadata: {
                capability_workbench: 'true',
                ...(guide.action === 'uppercase' ? { capability_workbench_uppercase: 'true' } : {}),
              },
            },
            signal: abort.signal,
          })
          update(id, { requestId: response.headers.get('x-gateway-request-id') })
          const result = await consumeModelResponse(response, {
            onDelta(delta) {
              if (!abort.signal.aborted)
                update(id, { output: (runs.value[id]?.output ?? '') + delta, chunks: (runs.value[id]?.chunks ?? 0) + 1 })
            },
          })
          update(id, { model: result.model, inputTokens: result.inputTokens, outputTokens: result.outputTokens, totalTokens: result.totalTokens })
          if (guide.action === 'uppercase' && runs.value[id]?.output !== text.toUpperCase())
            throw new Error('模型已返回，但大写转换未生效。请在当前配置的“生效请求”中开启“请求中间件 · 请求开始”，并将所选 Key 纳入范围。')
          if (guide.action === 'request' && result.model !== 'demo-echo')
            throw new Error('模型已返回，但路由未生效。请在当前配置的“生效请求”中开启模型路由，并将所选 Key 纳入范围。')
        }
      }
      if (!abort.signal.aborted)
        update(id, { phase: 'done', durationMs: Math.round(performance.now() - startedAt) })
    }
    catch (cause) {
      if (!abort.signal.aborted)
        update(id, { phase: 'error', error: cause instanceof Error ? cause.message : '操作失败，请重试' })
    }
    finally {
      if (!disposed) {
        if (abort.signal.aborted)
          update(id, { phase: 'stopped' })
        await refresh()
        activeId.value = ''
        controller = undefined
      }
    }
  }

  function stop(): void {
    controller?.abort()
  }

  onBeforeUnmount(() => {
    disposed = true
    stop()
  })

  return { selectedId, keyId, message, examples, selected, run, activeId, keyOptions, evidence, execute, stop }
}
