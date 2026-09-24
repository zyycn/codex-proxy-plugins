import type { Evidence, WorkbenchExample, WorkbenchKey, WorkbenchSnapshot } from '../types/workbench'
import { exampleStatuses } from '@/constants/workbench'
import { nonNegativeInteger, nullableString, record, string, stringArray } from '../validation'

function parseKey(value: unknown): WorkbenchKey {
  const item = record(value, 'Key')
  if (typeof item.enabled !== 'boolean')
    throw new Error('Key 状态格式无效')
  return { id: string(item.id, 'Key ID'), name: string(item.name, 'Key 名称'), enabled: item.enabled }
}

function parseEvidence(value: unknown): Evidence {
  const item = record(value, '执行事实')
  if (item.outcome !== 'passed' && item.outcome !== 'failed')
    throw new Error('执行事实结果格式无效')
  return {
    id: string(item.id, '事实 ID'),
    capability: string(item.capability, '能力'),
    event: string(item.event, '事件'),
    outcome: item.outcome,
    occurredAtMs: nonNegativeInteger(item.occurredAtMs, '事件时间'),
    requestId: nullableString(item.requestId, '请求 ID'),
    provider: nullableString(item.provider, 'Provider'),
    accountId: nullableString(item.accountId, '账号 ID'),
    model: nullableString(item.model, '模型'),
    details: record(item.details, '事实详情'),
  }
}

function parseExample(value: unknown): WorkbenchExample {
  const item = record(value, '基础示例')
  if (typeof item.status !== 'string' || !exampleStatuses.has(item.status))
    throw new Error('基础示例状态格式无效')
  return {
    id: string(item.id, '示例 ID'),
    title: string(item.title, '示例标题'),
    capabilities: stringArray(item.capabilities, '能力列表'),
    status: item.status as WorkbenchExample['status'],
    runCount: nonNegativeInteger(item.runCount, '运行次数'),
    lastEvidenceId: nullableString(item.lastEvidenceId, '事实 ID'),
    trigger: string(item.trigger, '触发方式'),
  }
}

export function parseSnapshot(input: unknown): WorkbenchSnapshot {
  const value = record(input, '工作台快照')
  const facts = record(value.facts, '执行事实')
  const latest = record(facts.latest, '最新执行事实')
  if (value.contractVersion !== 1)
    throw new Error('工作台合同版本不受支持')
  if (!Array.isArray(value.keys) || !Array.isArray(value.examples) || !Array.isArray(facts.entries))
    throw new Error('工作台快照格式无效')
  return {
    contractVersion: 1,
    keys: value.keys.map(parseKey),
    keysNextCursor: nullableString(value.keysNextCursor, 'Key 游标'),
    examples: value.examples.map(parseExample),
    facts: {
      latest: Object.fromEntries(Object.entries(latest).map(([key, item]) => [key, parseEvidence(item)])),
      entries: facts.entries.map(parseEvidence),
    },
  }
}
