// @env browser
import { isRecord } from '../../utils/guards'
import { consumeResponseDocument, outputText } from '../../utils/responses'
import { modelResponses } from './models'

export type ImageContextItem = Record<string, unknown>

export interface ImageRequest {
  clientKeyId: string
  model: string
  prompt: string
  quality: string
  size: string
  history: ImageContextItem[]
  image?: string
  signal: AbortSignal
  onTextDelta: (delta: string) => void
}

export interface ImageReply {
  input: ImageContextItem
  output: ImageContextItem[]
  imageUrl: string
  text: string
}

export function imageRequestBody(input: ImageRequest) {
  const content: ImageContextItem[] = [{ type: 'input_text', text: input.prompt }]
  if (input.image)
    content.push({ type: 'input_image', image_url: input.image })
  const message = { role: 'user', content }
  const body = {
    model: input.model,
    instructions: 'Help the user create and iteratively edit images in this conversation. When the user describes a scene or asks for visual changes, use the image generation tool to create or edit exactly one image. Follow the latest instruction while retaining earlier constraints unless changed. Answer questions conversationally without generating an image unless requested. Reply in the user’s language.',
    input: [...input.history, message],
    tools: [{ type: 'image_generation', model: 'gpt-image-2', action: 'auto', quality: input.quality, size: input.size, output_format: 'png' }],
    tool_choice: 'auto',
    parallel_tool_calls: false,
    include: ['reasoning.encrypted_content'],
    stream: true,
    store: false,
  }
  // 管理模型桥正文上限为 8 MiB，不能悄悄丢掉历史后仍声称在续聊。
  if (new TextEncoder().encode(JSON.stringify(body)).byteLength > 8 * 1024 * 1024)
    throw new Error('会话内容已超过 8 MiB，请新建对话，需要时附上要继续修改的图片')
  return { body, message }
}

export async function sendImageMessage(input: ImageRequest): Promise<ImageReply> {
  const { body, message } = imageRequestBody(input)
  const response = await modelResponses({ clientKeyId: input.clientKeyId, signal: input.signal, body })
  const streamedOutput: ImageContextItem[] = []
  const document = await consumeResponseDocument(response, {
    onTextDelta: input.onTextDelta,
    // Codex 的完成事件可能省略 output，保留 done 项及原顺序供下一轮重放。
    onOutputItem: item => streamedOutput.push(item),
  })
  const output = Array.isArray(document.output) && document.output.length
    ? document.output.filter(isRecord)
    : streamedOutput
  const image = output.find(item => item.type === 'image_generation_call' && typeof item.result === 'string' && item.result)
  const text = outputText({ ...document, output })
  if (!image && !text)
    throw new Error('模型没有返回图片或文字，请重试')
  return { input: message, output, imageUrl: image ? imageDataUrl(image.result as string) : '', text }
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
  // 隔离页只允许 data: 图片；校验格式后构造固定 MIME 的地址。
  if (![137, 80, 78, 71, 13, 10, 26, 10].every((byte, index) => decoded.charCodeAt(index) === byte))
    throw new Error('模型没有返回预期的 PNG 图片')
  return `data:image/png;base64,${result}`
}
