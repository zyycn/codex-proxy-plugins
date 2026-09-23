import type { ConsumeOptions, ModelResult } from '../types'
import { isRecord } from './guards'

function nullableNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : null
}

function errorFromPayload(value: unknown, fallback: string): Error {
  if (isRecord(value)) {
    if (typeof value.message === 'string')
      return new Error(value.message)
    if (isRecord(value.error) && typeof value.error.message === 'string')
      return new Error(value.error.message)
    if (isRecord(value.response) && isRecord(value.response.error) && typeof value.response.error.message === 'string')
      return new Error(value.response.error.message)
  }
  return new Error(fallback)
}

function outputText(value: Record<string, unknown>): string {
  if (typeof value.output_text === 'string')
    return value.output_text
  if (!Array.isArray(value.output))
    return ''

  const parts: string[] = []
  for (const rawItem of value.output) {
    if (!isRecord(rawItem) || !Array.isArray(rawItem.content))
      continue
    for (const rawContent of rawItem.content) {
      if (isRecord(rawContent) && rawContent.type === 'output_text' && typeof rawContent.text === 'string')
        parts.push(rawContent.text)
    }
  }
  return parts.join('')
}

function resultFromResponse(value: unknown, streamedText: string): ModelResult {
  if (!isRecord(value))
    throw new Error('模型返回的响应格式无效')
  if (value.status === 'error' || value.status === 'failed' || value.status === 'incomplete'
    || value.status === 'cancelled' || value.status === 'canceled') {
    throw errorFromPayload(value, '模型生成未完成')
  }
  const usage = isRecord(value.usage) ? value.usage : {}
  return {
    text: streamedText || outputText(value),
    responseId: typeof value.id === 'string' ? value.id : null,
    model: typeof value.model === 'string' ? value.model : null,
    inputTokens: nullableNumber(usage.input_tokens),
    outputTokens: nullableNumber(usage.output_tokens),
    totalTokens: nullableNumber(usage.total_tokens),
  }
}

async function responseError(response: Response): Promise<Error> {
  let text = ''
  try {
    text = await response.text()
  }
  catch {
    return new Error(`模型请求失败（HTTP ${response.status}）`)
  }
  try {
    return errorFromPayload(JSON.parse(text), `模型请求失败（HTTP ${response.status}）`)
  }
  catch {
    return new Error(text.trim() || `模型请求失败（HTTP ${response.status}）`)
  }
}

function eventData(block: string): string | null {
  const lines = block.split(/\r?\n/)
  const values = lines
    .filter(line => line.startsWith('data:'))
    .map(line => line.slice(5).replace(/^ /, ''))
  return values.length > 0 ? values.join('\n') : null
}

export async function consumeModelResponse(response: Response, options: ConsumeOptions): Promise<ModelResult> {
  if (!response.ok)
    throw await responseError(response)

  const type = response.headers.get('content-type')?.split(';', 1)[0]?.trim().toLowerCase()
  if (type === 'application/json' || type?.endsWith('+json')) {
    const value: unknown = await response.json()
    const result = resultFromResponse(value, '')
    if (result.text)
      options.onDelta(result.text)
    return result
  }
  if (type !== 'text/event-stream')
    throw new Error('模型返回了不支持的内容类型')
  if (!response.body)
    throw new Error('模型没有返回响应正文')

  const reader = response.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  let text = ''
  let completed: unknown

  function consumeBlock(block: string) {
    const data = eventData(block)
    if (!data || data === '[DONE]')
      return

    let event: unknown
    try {
      event = JSON.parse(data)
    }
    catch {
      throw new Error('模型返回了无效的流事件')
    }
    if (!isRecord(event) || typeof event.type !== 'string')
      throw new Error('模型返回了无效的流事件')

    if (event.type === 'response.output_text.delta') {
      if (typeof event.delta !== 'string')
        throw new Error('模型返回了无效的文本增量')
      text += event.delta
      options.onDelta(event.delta)
      return
    }
    if (event.type === 'response.completed') {
      completed = event.response
      return
    }
    if (event.type === 'response.failed' || event.type === 'response.incomplete' || event.type === 'error')
      throw errorFromPayload(event, '模型生成未完成')
  }

  try {
    while (true) {
      const { done, value } = await reader.read()
      buffer += decoder.decode(value, { stream: !done })

      let boundary = buffer.search(/\r?\n\r?\n/)
      while (boundary >= 0) {
        const separator = buffer.match(/\r?\n\r?\n/)?.[0] ?? '\n\n'
        const block = buffer.slice(0, boundary)
        buffer = buffer.slice(boundary + separator.length)
        consumeBlock(block)
        boundary = buffer.search(/\r?\n\r?\n/)
      }

      if (done)
        break
    }
    if (buffer.trim())
      consumeBlock(buffer)
  }
  catch (cause) {
    try {
      await reader.cancel(cause)
    }
    catch {
      // 上游可能已经以错误终止，取消失败不覆盖原始解析错误。
    }
    throw cause
  }
  finally {
    try {
      reader.releaseLock()
    }
    catch {
      // 已取消或终止的流可能不再持有 reader 锁。
    }
  }

  if (completed === undefined)
    throw new Error('模型流在完成事件前结束')
  return resultFromResponse(completed, text)
}
