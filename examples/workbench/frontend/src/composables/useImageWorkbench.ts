// @env browser
import type { Ref } from 'vue'
import type { WorkbenchSnapshot } from '../api'
import type { ImageTurn } from '../types'
import type { ImagePhase } from '../utils/imageConversation'
import { computed, onScopeDispose, shallowRef } from 'vue'
import { ImageConversation } from '../utils/imageConversation'
import { useModelCatalog } from './useModelCatalog'

const MAX_IMAGE_BYTES = 4 * 1024 * 1024

export function useImageWorkbench(snapshot: Ref<WorkbenchSnapshot | undefined>) {
  const catalog = useModelCatalog(snapshot)
  const prompt = shallowRef('')
  const quality = shallowRef('low')
  const size = shallowRef('1024x1024')
  const attachment = shallowRef('')
  const attachmentName = shallowRef('')
  const turns = shallowRef<ImageTurn[]>([])
  const selectedId = shallowRef<number | null>(null)
  const running = shallowRef(false)
  const reading = shallowRef(false)
  const error = shallowRef('')
  const warning = shallowRef('')
  const phase = shallowRef<ImagePhase>('preparing')
  const images = computed(() => turns.value.filter(turn => turn.status === 'complete' && turn.imageUrl))
  const selectedImage = computed(() => images.value.find(turn => turn.id === selectedId.value) ?? images.value.at(-1))
  const canSend = computed(() => !running.value && !reading.value && !catalog.loading.value
    && Boolean(catalog.clientKeyId.value && catalog.modelId.value && prompt.value.trim()))
  const conversation = new ImageConversation()
  let sequence = 0
  let controller: AbortController | undefined
  let fileReader: FileReader | undefined
  let disposed = false

  function clearAttachment() {
    fileReader?.abort()
    fileReader = undefined
    reading.value = false
    attachment.value = ''
    attachmentName.value = ''
  }

  async function selectFile(file?: File) {
    if (!file || running.value)
      return
    error.value = ''
    if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type) || !file.size || file.size > MAX_IMAGE_BYTES) {
      error.value = '请选择不超过 4 MiB 的 PNG、JPEG 或 WebP 图片'
      return
    }
    fileReader?.abort()
    const reader = new FileReader()
    fileReader = reader
    reading.value = true
    try {
      const data = await new Promise<string>((resolve, reject) => {
        reader.onload = () => typeof reader.result === 'string' ? resolve(reader.result) : reject(new Error('图片读取失败'))
        reader.onerror = () => reject(new Error('图片读取失败'))
        reader.onabort = () => reject(new DOMException('已取消', 'AbortError'))
        reader.readAsDataURL(file)
      })
      if (!disposed && fileReader === reader) {
        attachment.value = data
        attachmentName.value = file.name
      }
    }
    catch (cause) {
      if (!disposed && fileReader === reader && !(cause instanceof DOMException && cause.name === 'AbortError'))
        error.value = '图片读取失败，请重新选择'
    }
    finally {
      if (fileReader === reader) {
        reading.value = false
        fileReader = undefined
      }
    }
  }

  function updateTurn(id: number, patch: Partial<ImageTurn>) {
    turns.value = turns.value.map(turn => turn.id === id ? { ...turn, ...patch } : turn)
  }

  async function run(turn: ImageTurn) {
    const current = new AbortController()
    controller = current
    running.value = true
    error.value = ''
    try {
      const key = catalog.clientKeyId.value
      const model = catalog.modelId.value
      const reply = await conversation.send({
        clientKeyId: key,
        model,
        prompt: turn.prompt,
        quality: quality.value,
        size: size.value,
        image: turn.attachment || undefined,
        signal: current.signal,
        onPhase(value) { phase.value = value },
        onTextDelta(delta) {
          if (!disposed && !current.signal.aborted) {
            const text = turns.value.find(item => item.id === turn.id)?.text ?? ''
            updateTurn(turn.id, { text: text + delta })
          }
        },
      })
      if (disposed || current.signal.aborted)
        return
      warning.value = conversation.recoveryWarning
      updateTurn(turn.id, { status: 'complete', text: reply.text, imageUrl: reply.imageUrl })
      if (reply.imageUrl)
        selectedId.value = turn.id
    }
    catch (cause) {
      if (!disposed && !current.signal.aborted)
        updateTurn(turn.id, { status: 'error', error: cause instanceof Error ? cause.message : '请求失败，请重试' })
    }
    finally {
      if (controller === current) {
        running.value = false
        controller = undefined
      }
    }
  }

  function send() {
    if (!canSend.value)
      return
    const turn: ImageTurn = {
      id: ++sequence,
      prompt: prompt.value.trim(),
      attachment: attachment.value,
      attachmentName: attachmentName.value,
      status: 'running',
      text: '',
      imageUrl: '',
      error: '',
    }
    turns.value = [...turns.value, turn]
    prompt.value = ''
    clearAttachment()
    void run(turn)
  }

  function retry() {
    const turn = turns.value.at(-1)
    if (running.value || reading.value || catalog.loading.value || !turn || !['error', 'cancelled'].includes(turn.status))
      return
    updateTurn(turn.id, { status: 'running', text: '', error: '' })
    void run(turn)
  }

  function stop() {
    controller?.abort()
    const turn = turns.value.at(-1)
    if (turn?.status === 'running')
      updateTurn(turn.id, { status: 'cancelled' })
  }

  function newConversation() {
    if (running.value)
      return
    clearAttachment()
    conversation.reset()
    turns.value = []
    selectedId.value = null
    prompt.value = ''
    error.value = ''
    warning.value = ''
  }

  onScopeDispose(() => {
    disposed = true
    controller?.abort()
    fileReader?.abort()
  })

  return {
    ...catalog,
    modelsError: catalog.error,
    prompt,
    quality,
    size,
    attachment,
    attachmentName,
    turns,
    images,
    selectedId,
    selectedImage,
    running,
    reading,
    error,
    warning,
    phase,
    canSend,
    selectFile,
    clearAttachment,
    send,
    retry,
    stop,
    newConversation,
  }
}
