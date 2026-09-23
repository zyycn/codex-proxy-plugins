import type { PreparedAccount } from '../types/workbench'
import request from '../request'
import { record, string } from '../validation'

export async function prepareDemoAccounts(): Promise<PreparedAccount[]> {
  const reply = await request({
    url: 'api/demo-accounts/prepare',
    method: 'POST',
    data: {},
  })
  const value = record(reply, '演示账号')
  if (!Array.isArray(value.accounts))
    throw new Error('演示账号格式无效')
  return value.accounts.map((raw) => {
    const item = record(raw, '演示账号')
    if (typeof item.created !== 'boolean')
      throw new Error('演示账号状态格式无效')
    return { id: string(item.id, '账号 ID'), name: string(item.name, '账号名称'), created: item.created }
  })
}
