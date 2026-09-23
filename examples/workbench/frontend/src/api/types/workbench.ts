export type TaskKind = 'summarize' | 'translate' | 'rewrite'
export type SourceKind = 'text' | 'url'
export type ExampleStatus = 'not_run' | 'passed' | 'failed' | 'pending'

export interface WorkbenchKey {
  id: string
  name: string
  enabled: boolean
}

export interface Evidence {
  id: string
  capability: string
  event: string
  outcome: 'passed' | 'failed'
  occurredAtMs: number
  requestId: string | null
  provider: string | null
  accountId: string | null
  model: string | null
  details: Record<string, unknown>
}

export interface WorkbenchExample {
  id: string
  title: string
  capabilities: string[]
  status: ExampleStatus
  runCount: number
  lastEvidenceId: string | null
  trigger: string
}

export interface WorkbenchSnapshot {
  contractVersion: 1
  provider: {
    id: 'demo'
    models: string[]
  }
  keys: WorkbenchKey[]
  keysNextCursor: string | null
  examples: WorkbenchExample[]
  facts: {
    latest: Record<string, Evidence>
    entries: Evidence[]
  }
}

export interface SavedTask {
  id: string
  title: string
  sourceKind: SourceKind
  source: string
  sourceUrl: string | null
  task: TaskKind
  instruction: string
  result: string
  modelId: string
  clientKeyId: string
  updatedAt: number
}

export interface SavedTasks {
  selectedId: string | null
  entries: SavedTask[]
}

export interface SavedTasksReply {
  version: number | null
  value: SavedTasks | null
}

export interface FetchTextReply {
  url: string
  status: number
  contentType: string | null
  text: string
  truncated: boolean
  bytes: number
}

export interface PreparedAccount {
  id: string
  name: string
  created: boolean
}
