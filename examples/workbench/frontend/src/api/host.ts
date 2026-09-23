// @env browser
import type { PluginHost } from './types/host'

export function getHost(): PluginHost {
  const host = window.codexProxyPlugin
  if (!host || host.version !== 2)
    throw new Error('请从网关的插件管理页面打开')
  return host
}
