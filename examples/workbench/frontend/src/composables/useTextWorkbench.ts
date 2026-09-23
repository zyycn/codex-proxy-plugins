import type { ComputedRef } from 'vue'
// @env browser
import type { FetchTextReply, SavedTask, SourceKind, TaskKind } from '../api'
import type { GenerationPhase, ResponseFacts, UseTextWorkbenchOptions } from '../types'
import { toast } from '@codex-proxy/ui'
import { computed, onBeforeUnmount, onMounted, readonly, shallowReadonly, shallowRef, watch } from 'vue'
import { fetchText, modelResponses } from '../api'
import { createOperationEpoch } from '../utils/operationEpoch'
import { consumeModelResponse } from '../utils/responses'
import {
  buildContinuationRequest,
  buildInitialRequest,
  instructionFor,
  savedTaskTitle,
} from '../utils/workbench'
import { useModelCatalog } from './useModelCatalog'
import { useTaskHistory } from './useTaskHistory'

export function useTextWorkbench(options: UseTextWorkbenchOptions) {
  const catalog = useModelCatalog(options.snapshot)
  const { clientKeyId, modelId, keyOptions, modelOptions, loading: modelsLoading, error: modelsError } = catalog
  const history = useTaskHistory()
  const { loading: historyLoading, saving: historySaving, error: historyError, options: historyOptions } = history

  const sourceKind = shallowRef<SourceKind>('text')
  const source = shallowRef('')
  const sourceUrl = shallowRef('')
  const task = shallowRef<TaskKind>('summarize')
  const instruction = shallowRef('')
  const result = shallowRef('')
  const phase = shallowRef<GenerationPhase>('idle')
  const facts = shallowRef<ResponseFacts>()
  const fetchReply = shallowRef<FetchTextReply>()
  const fetchingText = shallowRef(false)
  const activeController = shallowRef<AbortController>()

  const selectedTaskId = shallowRef('')
  const taskEpoch = createOperationEpoch()
  const generationEpoch = createOperationEpoch()
  const fetchEpoch = createOperationEpoch()
  let historyInitialized = false
  let skipNextHistoryRestore = false

  const running = computed(() => phase.value === 'requesting' || phase.value === 'streaming')
  const canGenerate = computed(() => Boolean(
    source.value.trim() && clientKeyId.value && modelId.value && !running.value && !fetchingText.value,
  ))
  const defaultInstruction: ComputedRef<string> = computed(() => instructionFor(task.value, ''))

  watch(selectedTaskId, (id) => {
    if (skipNextHistoryRestore) {
      skipNextHistoryRestore = false
      return
    }
    if (!id) {
      newTask()
      return
    }
    const item = history.tasks.value.entries.find(entry => entry.id === id)
    if (!item)
      return
    restoreTask(item)
  }, { flush: 'sync' })

  async function loadHistory() {
    const currentTask = taskEpoch.current()
    const loaded = await history.load()
    if (!loaded)
      return
    if (!historyInitialized && taskEpoch.isCurrent(currentTask))
      selectedTaskId.value = loaded.selectedId ?? ''
    historyInitialized = true
  }

  function restoreTask(item: SavedTask) {
    history.clearFeedback()
    taskEpoch.advance()
    fetchEpoch.advance()
    stop()
    fetchingText.value = false
    fetchReply.value = undefined
    selectedTaskId.value = item.id
    sourceKind.value = item.sourceKind
    source.value = item.source
    sourceUrl.value = item.sourceUrl ?? ''
    task.value = item.task
    instruction.value = item.instruction
    result.value = item.result
    clientKeyId.value = item.clientKeyId
    modelId.value = item.modelId
    phase.value = item.result ? 'done' : 'idle'
    facts.value = undefined
  }

  function newTask() {
    history.clearFeedback()
    if (selectedTaskId.value) {
      selectedTaskId.value = ''
      return
    }
    taskEpoch.advance()
    fetchEpoch.advance()
    stop()
    fetchingText.value = false
    sourceKind.value = 'text'
    source.value = ''
    sourceUrl.value = ''
    task.value = 'summarize'
    instruction.value = ''
    result.value = ''
    phase.value = 'idle'
    fetchReply.value = undefined
    facts.value = undefined
  }

  async function saveCurrent() {
    if (historySaving.value || !source.value.trim() || !result.value.trim())
      return
    const id = selectedTaskId.value || crypto.randomUUID()
    const currentTask = taskEpoch.current()
    const entry: SavedTask = {
      id,
      title: savedTaskTitle(source.value, task.value),
      sourceKind: sourceKind.value,
      source: source.value,
      sourceUrl: sourceKind.value === 'url' ? (sourceUrl.value.trim() || null) : null,
      task: task.value,
      instruction: instruction.value,
      result: result.value,
      modelId: modelId.value,
      clientKeyId: clientKeyId.value,
      updatedAt: Date.now(),
    }
    if (await history.save(entry) && taskEpoch.isCurrent(currentTask) && selectedTaskId.value !== id) {
      skipNextHistoryRestore = true
      selectedTaskId.value = id
    }
  }

  async function loadFromUrl() {
    if (fetchingText.value || running.value || !sourceUrl.value.trim())
      return
    fetchingText.value = true
    fetchReply.value = undefined
    const currentTask = taskEpoch.current()
    const currentFetch = fetchEpoch.advance()
    const requestedUrl = sourceUrl.value.trim()
    try {
      const reply = await fetchText(requestedUrl)
      if (!taskEpoch.isCurrent(currentTask) || !fetchEpoch.isCurrent(currentFetch)
        || sourceKind.value !== 'url' || sourceUrl.value.trim() !== requestedUrl) {
        return
      }
      fetchReply.value = reply
      source.value = reply.text
      sourceUrl.value = reply.url
    }
    catch (cause) {
      if (!taskEpoch.isCurrent(currentTask) || !fetchEpoch.isCurrent(currentFetch)
        || sourceKind.value !== 'url' || sourceUrl.value.trim() !== requestedUrl) {
        return
      }
      toast.error(cause instanceof Error ? cause.message : '读取网页失败，请稍后重试')
    }
    finally {
      if (taskEpoch.isCurrent(currentTask) && fetchEpoch.isCurrent(currentFetch))
        fetchingText.value = false
    }
  }

  async function run(body: Record<string, unknown>) {
    if (running.value)
      return
    history.clearFeedback()
    const controller = new AbortController()
    const currentTask = taskEpoch.current()
    const currentGeneration = generationEpoch.advance()
    const isCurrent = () => taskEpoch.isCurrent(currentTask)
      && generationEpoch.isCurrent(currentGeneration)
      && activeController.value === controller
    activeController.value = controller
    facts.value = undefined
    result.value = ''
    phase.value = 'requesting'
    const startedAt = performance.now()
    let requestId: string | null = null
    try {
      const response = await modelResponses({ clientKeyId: clientKeyId.value, body, signal: controller.signal })
      if (!isCurrent()) {
        await response.body?.cancel().catch(() => undefined)
        return
      }
      requestId = response.headers.get('x-gateway-request-id')
      phase.value = 'streaming'
      const modelResult = await consumeModelResponse(response, {
        onDelta(delta) {
          if (isCurrent())
            result.value += delta
        },
      })
      if (!isCurrent())
        return
      phase.value = 'done'
      await options.refreshSnapshot()
      if (!isCurrent())
        return
      const evidence = options.snapshot.value?.facts.entries.filter(item =>
        requestId !== null && item.requestId === requestId,
      ) ?? []
      facts.value = {
        requestId,
        responseId: modelResult.responseId,
        model: modelResult.model,
        inputTokens: modelResult.inputTokens,
        outputTokens: modelResult.outputTokens,
        totalTokens: modelResult.totalTokens,
        durationMs: Math.round(performance.now() - startedAt),
        evidence,
      }
    }
    catch (cause) {
      if (!isCurrent())
        return
      if (controller.signal.aborted) {
        phase.value = 'stopped'
      }
      else {
        phase.value = 'error'
        toast.error(cause instanceof Error ? cause.message : '生成失败，请重试')
      }
      await options.refreshSnapshot()
      if (!isCurrent())
        return
    }
    finally {
      if (isCurrent())
        activeController.value = undefined
    }
  }

  function generate() {
    if (!canGenerate.value)
      return
    void run(buildInitialRequest(modelId.value, task.value, source.value.trim(), instruction.value))
  }

  function continueWith(adjustment: string) {
    if (!canGenerate.value || !result.value.trim() || !adjustment.trim())
      return
    const previousResult = result.value
    void run(buildContinuationRequest(
      modelId.value,
      task.value,
      source.value.trim(),
      instruction.value,
      previousResult,
      adjustment.trim(),
    ))
  }

  function retry() {
    if (!source.value.trim() || !clientKeyId.value || !modelId.value)
      return
    generate()
  }

  function stop() {
    const wasRunning = running.value
    generationEpoch.advance()
    activeController.value?.abort()
    activeController.value = undefined
    if (wasRunning)
      phase.value = 'stopped'
  }

  onMounted(loadHistory)
  onBeforeUnmount(() => {
    taskEpoch.advance()
    fetchEpoch.advance()
    stop()
  })

  return {
    clientKeyId,
    modelId,
    keyOptions,
    modelOptions,
    modelsLoading: readonly(modelsLoading),
    modelsError: readonly(modelsError),
    sourceKind,
    source,
    sourceUrl,
    task,
    instruction,
    defaultInstruction,
    result,
    phase: readonly(phase),
    running,
    facts: shallowReadonly(facts),
    fetchReply: shallowReadonly(fetchReply),
    fetchingText: readonly(fetchingText),
    historyOptions,
    selectedTaskId,
    historyLoading: readonly(historyLoading),
    historySaving: readonly(historySaving),
    historyError,
    historyNotice: history.notice,
    canGenerate,
    loadFromUrl,
    generate,
    continueWith,
    retry,
    stop,
    saveCurrent,
    loadHistory,
    newTask,
  }
}
