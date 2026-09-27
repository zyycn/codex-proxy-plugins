import type { ImageContextItem, ImageRequest, ImageSettings } from '../api/modules/images'
import { encodedBytes, MAX_REQUEST_BYTES, sendImageRequest } from '../api/modules/images'
import { ModelResponseError } from './responses'

export type ImagePhase = 'preparing' | 'restoring' | 'generating'

interface Message extends ImageSettings {
  prompt: string
  image?: string
  signal: AbortSignal
  onTextDelta: (delta: string) => void
  onPhase: (phase: ImagePhase) => void
}

// 页面拥有完整历史，宿主拥有账号绑定与上游连接；response ID 只作为可失效的续接句柄。
export class ImageConversation {
  private history: ImageContextItem[] = []
  private historyBytes = 2
  private settings?: ImageSettings
  private responseId?: string
  private busy = false

  get recoveryWarning(): string {
    return this.historyBytes >= MAX_REQUEST_BYTES - 16 * 1024
      ? '历史较大，连接失效或更改图片设置后可能无法恢复；建议适时新建对话'
      : ''
  }

  reset() {
    if (this.busy)
      throw new Error('请等待当前请求结束')
    this.history = []
    this.historyBytes = 2
    this.settings = undefined
    this.responseId = undefined
  }

  async send(message: Message) {
    if (this.busy)
      throw new Error('当前对话正在生成，请稍候')
    if (this.settings && (this.settings.clientKeyId !== message.clientKeyId || this.settings.model !== message.model))
      throw new Error('会话所用 Key 或模型已变更，请新建对话')
    // 限制页面驻留量，不通过截断历史悄悄改变对话含义。
    if (this.historyBytes >= 32 * 1024 * 1024)
      throw new Error('会话历史已达到示例的 32 MiB 保留上限，请新建对话')
    this.busy = true
    let hasOutput = false
    const content: ImageContextItem[] = [{ type: 'input_text', text: message.prompt }]
    if (message.image)
      content.push({ type: 'input_image', image_url: message.image })
    const input = { role: 'user', content }
    const request: ImageRequest = {
      ...message,
      input: [input],
      onTextDelta(delta) {
        hasOutput = true
        message.onTextDelta(delta)
      },
      onOutputItem() { hasOutput = true },
    }

    const prepare = async () => {
      message.onPhase(this.history.length ? 'restoring' : 'preparing')
      // generate:false 建立可接续的 WebSocket 上下文，不执行本轮生图。
      const warmup = await sendImageRequest({ ...request, input: [...this.history, input], warmup: true, onTextDelta: undefined, onOutputItem: undefined })
      return { ...request, input: [], previousResponseId: warmup.responseId }
    }

    try {
      const sameSettings = this.settings?.quality === message.quality && this.settings.size === message.size
      let next = this.responseId && sameSettings ? { ...request, previousResponseId: this.responseId } : await prepare()
      message.onPhase('generating')
      let reply
      try {
        reply = await sendImageRequest(next)
      }
      catch (cause) {
        // 只自动恢复明确缺失的句柄，且尚未出现语义输出；超时、取消和普通失败不能重复计费。
        if (message.signal.aborted || hasOutput || !(cause instanceof ModelResponseError) || cause.code !== 'previous_response_not_found')
          throw cause
        next = await prepare()
        message.onPhase('generating')
        reply = await sendImageRequest(next)
      }
      message.signal.throwIfAborted()
      this.historyBytes += encodedBytes([input, ...reply.output]) - 1
      this.history.push(input, ...reply.output)
      this.settings = { clientKeyId: message.clientKeyId, model: message.model, quality: message.quality, size: message.size }
      this.responseId = reply.responseId
      return reply
    }
    catch (cause) {
      // 失败后上游可能已前进；下次手动重试从已提交历史建立新链。
      this.responseId = undefined
      throw cause
    }
    finally {
      this.busy = false
    }
  }
}
