// @env browser
import { isRecord } from '../../utils/guards'
import { consumeResponseDocument, outputText } from '../../utils/responses'
import { modelResponses } from './models'

export type ImageContextItem = Record<string, unknown>
export const MAX_REQUEST_BYTES = 8 * 1024 * 1024

export interface ImageSettings {
  clientKeyId: string
  model: string
  quality: string
  size: string
}

export interface ImageRequest extends ImageSettings {
  input: ImageContextItem[]
  previousResponseId?: string
  warmup?: boolean
  signal: AbortSignal
  onTextDelta?: (delta: string) => void
  onOutputItem?: () => void
}

export interface ImageReply {
  responseId: string
  output: ImageContextItem[]
  imageUrl: string
  text: string
}

export function imageRequestBody(input: ImageRequest) {
  return {
    model: input.model,
    instructions: 'Help the user create and iteratively edit images in this conversation. When the user describes a scene or asks for visual changes, use the image generation tool to create or edit exactly one image. Follow the latest instruction while retaining earlier constraints unless changed. Answer questions conversationally without generating an image unless requested. Reply in the user’s language.',
    input: input.input,
    ...(input.previousResponseId ? { previous_response_id: input.previousResponseId } : {}),
    ...(input.warmup ? { generate: false } : {}),
    tools: [{ type: 'image_generation', model: 'gpt-image-2', action: 'auto', quality: input.quality, size: input.size, output_format: 'png' }],
    tool_choice: 'auto',
    parallel_tool_calls: false,
    include: ['reasoning.encrypted_content'],
    stream: true,
    store: false,
  }
}

export function encodedBytes(value: unknown): number {
  return new TextEncoder().encode(JSON.stringify(value)).byteLength
}

export async function sendImageRequest(input: ImageRequest): Promise<ImageReply> {
  input.signal.throwIfAborted()
  const body = imageRequestBody(input)
  // 仅度量本次发送内容，正常续聊不序列化历史图片。
  if (encodedBytes(body) > MAX_REQUEST_BYTES)
    throw new Error(input.warmup ? '完整历史超过 8 MiB，无法恢复连接；请新建对话，可附上要继续修改的图片' : '本次消息超过 8 MiB，请减少附件或文字')
  const response = await modelResponses({ clientKeyId: input.clientKeyId, signal: input.signal, body })
  const streamedOutput: ImageContextItem[] = []
  const document = await consumeResponseDocument(response, {
    onTextDelta: input.onTextDelta,
    // Codex 的完成事件可能省略 output，恢复时仍需保留 done 项的原始顺序。
    onOutputItem(item) {
      streamedOutput.push(item)
      input.onOutputItem?.()
    },
  })
  input.signal.throwIfAborted()
  const output = Array.isArray(document.output) && document.output.length
    ? document.output.filter(isRecord)
    : streamedOutput
  const responseId = typeof document.id === 'string' ? document.id : ''
  if (!responseId)
    throw new Error('模型没有返回续聊所需的响应 ID')
  if (input.warmup) {
    if (output.length)
      throw new Error('模型没有按预期建立会话，请确认宿主与上游支持 generate: false')
    return { responseId, output, imageUrl: '', text: '' }
  }
  const image = output.find(item => item.type === 'image_generation_call' && typeof item.result === 'string' && item.result)
  const text = outputText({ ...document, output })
  if (!image && !text)
    throw new Error('模型没有返回图片或文字，请重试')
  return { responseId, output, imageUrl: image ? imageDataUrl(image.result as string) : '', text }
}

function imageDataUrl(result: string): string {
  if (result.length > 32 * 1024 * 1024)
    throw new Error('返回图片超过示例的大小限制')
  let decoded: string
  try {
    decoded = atob(result)
  }
  catch {
    throw new Error('模型返回了无效的图片数据')
  }
  if (![137, 80, 78, 71, 13, 10, 26, 10].every((byte, index) => decoded.charCodeAt(index) === byte))
    throw new Error('模型没有返回预期的 PNG 图片')
  return `data:image/png;base64,${result}`
}
