import type { FetchTextReply } from '../types/workbench'
import request from '../request'
import { nonNegativeInteger, nullableString, record, string } from '../validation'

export async function fetchText(url: string): Promise<FetchTextReply> {
  const reply = await request({
    url: 'api/fetch-text',
    method: 'POST',
    data: { url },
  })
  const value = record(reply, '取文结果')
  if (typeof value.truncated !== 'boolean')
    throw new Error('取文截断状态格式无效')
  return {
    url: string(value.url, '网页地址'),
    status: nonNegativeInteger(value.status, 'HTTP 状态'),
    contentType: nullableString(value.contentType, '内容类型'),
    text: string(value.text, '网页正文'),
    truncated: value.truncated,
    bytes: nonNegativeInteger(value.bytes, '正文字节数'),
  }
}
