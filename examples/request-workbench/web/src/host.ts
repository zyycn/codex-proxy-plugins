interface ManagementReply {
  status: number
  contentType: string
  body: ArrayBuffer
}

interface PluginHost {
  version: number
  request: (input: {
    method: 'GET' | 'POST'
    path: string
    contentType?: string
    body?: string
  }) => Promise<ManagementReply>
}

declare global {
  interface Window {
    codexProxyPlugin?: PluginHost
  }
}

/** 页面只能经当前实例的管理桥调用已声明路由，不读取宿主凭据。 */
export async function request(method: 'GET' | 'POST', path: string, body?: unknown): Promise<unknown> {
  const host = window.codexProxyPlugin
  if (!host || host.version !== 1)
    throw new Error('请从网关的插件管理页面打开')
  const reply = await host.request({
    method,
    path,
    ...(body === undefined ? {} : { contentType: 'application/json', body: JSON.stringify(body) }),
  })
  if (reply.status < 200 || reply.status >= 300)
    throw new Error(`插件请求失败（HTTP ${reply.status}）`)
  if (reply.contentType.split(';', 1)[0]?.trim().toLowerCase() !== 'application/json')
    throw new Error('插件返回了不支持的内容类型')
  return JSON.parse(new TextDecoder().decode(reply.body))
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}
