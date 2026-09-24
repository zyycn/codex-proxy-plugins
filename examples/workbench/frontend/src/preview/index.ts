// @env browser
import type { Evidence, PluginHost, SavedTasks, WorkbenchExample } from '../api'
import {
  applyResolvedTheme,
  DEFAULT_CUSTOM_THEME_COLOR,
  DEFAULT_THEME_COLOR,
  resolveTheme,
} from '@codex-proxy/ui/theme'

import { exampleDefinitions } from './data'

const encoder = new TextEncoder()

let sequence = 0
let taskVersion: number | null = null
let tasks: SavedTasks | null = null
const evidence: Evidence[] = []
const examples: WorkbenchExample[] = exampleDefinitions.map(([id, title, capabilities, trigger]) => ({
  id,
  title,
  capabilities: [...capabilities],
  status: 'not_run',
  runCount: 0,
  lastEvidenceId: null,
  trigger,
}))

function json(value: unknown, status = 200) {
  const bytes = encoder.encode(JSON.stringify(value))
  return Promise.resolve({
    status,
    contentType: 'application/json',
    body: bytes.buffer,
  })
}

function parseBody(body?: string): Record<string, unknown> {
  if (!body)
    return {}
  const value: unknown = JSON.parse(body)
  return value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : {}
}

function recordEvidence(
  capability: string,
  event: string,
  requestId: string | null = null,
  model: string | null = null,
) {
  const item: Evidence = {
    id: `preview-evidence-${++sequence}`,
    capability,
    event,
    outcome: 'passed',
    occurredAtMs: Date.now(),
    requestId,
    provider: model ? 'openai' : null,
    accountId: model ? 'preview-account' : null,
    model,
    details: {},
  }
  evidence.unshift(item)
  evidence.splice(64)
  for (const example of examples) {
    if (!example.capabilities.includes(capability))
      continue
    example.runCount += 1
    example.lastEvidenceId = item.id
    const recorded = example.capabilities.filter(expected => evidence.some(entry => entry.capability === expected)).length
    example.status = recorded === example.capabilities.length ? 'passed' : 'pending'
  }
}

function snapshot() {
  const latest: Record<string, Evidence> = {}
  for (const item of evidence) {
    if (!latest[item.capability])
      latest[item.capability] = item
  }
  return {
    contractVersion: 1,
    keys: [
      { id: 'preview-key-primary', name: '预览 Key', enabled: true },
      { id: 'preview-key-disabled', name: '已停用 Key', enabled: false },
    ],
    keysNextCursor: null,
    examples,
    facts: { latest, entries: evidence },
  }
}

function error(code: string, message: string, status = 400) {
  return json({ error: { code, message } }, status)
}

async function managementRequest(input: Parameters<PluginHost['request']>[0]) {
  await new Promise(resolve => window.setTimeout(resolve, 80))
  const body = parseBody(input.body)

  if (input.method === 'GET' && input.path === 'api/snapshot') {
    recordEvidence('management', 'workbench.snapshot')
    return json(snapshot())
  }
  if (input.method === 'POST' && input.path === 'api/echo') {
    if (typeof body.message !== 'string' || !body.message.trim())
      return error('invalid_message', '请输入 Echo 文本')
    recordEvidence('management', 'workbench.echo')
    return json({ message: body.message })
  }
  if (input.method === 'POST' && input.path === 'api/models') {
    recordEvidence('models', 'host.models.list')
    return json({ models: ['gpt-preview'] })
  }
  if (input.method === 'POST' && input.path === 'api/fetch-text') {
    if (typeof body.url !== 'string' || !body.url.trim())
      return error('invalid_url', '请输入网页地址')
    return json({
      url: body.url,
      status: 200,
      contentType: 'text/plain; charset=utf-8',
      text: '这是一段由独立预览提供的网页正文。它只用于检查取文状态、长文本布局和错误反馈，不会访问真实网络。',
      truncated: false,
      bytes: 132,
    })
  }
  if (input.method === 'GET' && input.path === 'api/tasks')
    return json({ version: taskVersion, value: tasks })
  if (input.method === 'POST' && input.path === 'api/tasks') {
    const expectedVersion = body.expectedVersion
    if (expectedVersion !== taskVersion)
      return error('version_conflict', '保存记录已在别处更新，请重新载入', 409)
    tasks = body.value as SavedTasks
    taskVersion = (taskVersion ?? 0) + 1
    return json({ version: taskVersion })
  }
  if (input.method === 'POST' && input.path === 'api/log')
    return json({ recorded: true })
  return error('route_not_found', '未知的管理路由', 404)
}

function abortError(): DOMException {
  return new DOMException('The operation was aborted', 'AbortError')
}

function delay(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(abortError())
      return
    }
    const timer = window.setTimeout(resolve, ms)
    signal?.addEventListener('abort', () => {
      window.clearTimeout(timer)
      reject(abortError())
    }, { once: true })
  })
}

async function modelResponse(input: Parameters<PluginHost['models']['responses']>[0]): Promise<Response> {
  await delay(140, input.signal)
  const model = typeof input.body.model === 'string' ? input.body.model : 'gpt-preview'
  const output = '这份预览结果提炼了原文的主要结论，并保留了关键事实与上下文。\n\n实际安装后，内容会由所选 Key 和模型通过普通 Responses 请求实时生成。'
  const chunks = output.match(/.{1,12}/gs) ?? [output]
  const requestId = `preview-request-${++sequence}`
  let index = 0

  const stream = new ReadableStream<Uint8Array>({
    async pull(controller) {
      try {
        await delay(70, input.signal)
        if (index < chunks.length) {
          const event = { type: 'response.output_text.delta', delta: chunks[index++] }
          controller.enqueue(encoder.encode(`data: ${JSON.stringify(event)}\n\n`))
          return
        }

        for (const capability of ['middleware', 'model_router', 'scheduler', 'request_lifecycle', 'usage'])
          recordEvidence(capability, `preview.${capability}`, requestId, model)

        const completed = {
          type: 'response.completed',
          response: {
            id: `preview-response-${sequence}`,
            model,
            output_text: output,
            usage: { input_tokens: 48, output_tokens: 36, total_tokens: 84 },
          },
        }
        controller.enqueue(encoder.encode(`data: ${JSON.stringify(completed)}\n\n`))
        controller.close()
      }
      catch (cause) {
        controller.error(cause)
      }
    },
  })

  return new Response(stream, {
    status: 200,
    headers: {
      'content-type': 'text/event-stream',
      'x-gateway-request-id': requestId,
    },
  })
}

export function installPreviewHost(): boolean {
  if (!import.meta.env.DEV)
    return false

  const parameters = new URLSearchParams(window.location.search)
  const theme = parameters.get('theme') === 'dark' ? 'dark' : 'light'
  applyResolvedTheme(
    document.documentElement,
    resolveTheme(theme, DEFAULT_THEME_COLOR, DEFAULT_CUSTOM_THEME_COLOR),
  )

  const value: PluginHost = {
    version: 2,
    theme,
    request: managementRequest,
    callbackTicket: async () => 'https://preview.invalid/callback',
    resourceUrl: path => `https://preview.invalid/${path}`,
    models: { responses: modelResponse },
  }
  Object.defineProperty(window, 'codexProxyPlugin', {
    configurable: true,
    value,
  })
  return true
}
