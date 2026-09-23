import request from '../request'
import { record, string } from '../validation'

export async function echo(message: string): Promise<string> {
  const reply = await request({
    url: 'api/echo',
    method: 'POST',
    data: { message },
  })
  return string(record(reply, 'Echo 结果').message, 'Echo 消息')
}
