import type { Component, Ref } from 'vue'
import type { Evidence, PreparedAccount, WorkbenchSnapshot } from '../api'

export type WorkbenchView = 'text' | 'examples'

export interface ResponseFacts {
  requestId: string | null
  responseId: string | null
  model: string | null
  inputTokens: number | null
  outputTokens: number | null
  totalTokens: number | null
  durationMs: number | null
  evidence: Evidence[]
}

export interface UiSelectOption {
  label: string
  value: string
  description?: string
  disabled?: boolean
}

export interface UiSegmentedOption {
  label: string
  value: string
  icon?: Component
  disabled?: boolean
}

export type ExampleAction = 'echo' | 'uppercase' | 'request' | 'accounts' | 'external'

export interface ExampleGuide {
  id: string
  title: string
  summary: string
  icon: Component
  group: 'interactive' | 'integration'
  action: ExampleAction
  actionLabel: string
  capabilities: string[]
  expected: string
  steps?: string[]
  command?: string
}

export interface ExampleRun {
  phase: 'running' | 'done' | 'stopped' | 'error'
  input: string
  output: string
  error: string
  accounts: PreparedAccount[]
  requestId: string | null
  model: string | null
  inputTokens: number | null
  outputTokens: number | null
  totalTokens: number | null
  chunks: number
  startedAtMs: number
  durationMs: number | null
}

export interface ModelResult {
  text: string
  responseId: string | null
  model: string | null
  inputTokens: number | null
  outputTokens: number | null
  totalTokens: number | null
}

export interface ConsumeOptions {
  onDelta: (delta: string) => void
}

export interface OperationEpoch {
  current: () => number
  advance: () => number
  isCurrent: (token: number) => boolean
}

export type GenerationPhase = 'idle' | 'requesting' | 'streaming' | 'done' | 'stopped' | 'error'

export interface UseTextWorkbenchOptions {
  snapshot: Ref<WorkbenchSnapshot | undefined>
  refreshSnapshot: () => Promise<void> | void
}
