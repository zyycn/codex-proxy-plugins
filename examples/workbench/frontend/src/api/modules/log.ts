import request from '../request'
import { record } from '../validation'

export async function recordSafeLog(): Promise<boolean> {
  const reply = await request({
    url: 'api/log',
    method: 'POST',
    data: {},
  })
  const value = record(reply, '日志结果')
  if (typeof value.recorded !== 'boolean')
    throw new Error('日志结果格式无效')
  return value.recorded
}
