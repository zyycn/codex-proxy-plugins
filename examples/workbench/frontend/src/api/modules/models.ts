import type { ModelRequestInput } from '../types/host'
import { getHost } from '../host'
import request from '../request'
import { record, stringArray } from '../validation'

export async function listModels(clientKeyId: string): Promise<string[]> {
  const reply = await request({
    url: 'api/models',
    method: 'POST',
    data: { clientKeyId },
  })
  return stringArray(record(reply, '模型目录').models, '模型目录')
}

// 模型桥交付原始 JSON / SSE，并将取消传递给宿主，不经过管理接口的 JSON 解析。
export function modelResponses(input: ModelRequestInput): Promise<Response> {
  return getHost().models.responses(input)
}
