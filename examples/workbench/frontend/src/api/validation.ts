import { isRecord } from '../utils/guards'

export function string(value: unknown, label: string): string {
  if (typeof value !== 'string')
    throw new Error(`${label}格式无效`)
  return value
}

export function nullableString(value: unknown, label: string): string | null {
  if (value === null)
    return null
  return string(value, label)
}

export function nonNegativeInteger(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value) || Number(value) < 0)
    throw new Error(`${label}格式无效`)
  return Number(value)
}

export function record(value: unknown, label: string): Record<string, unknown> {
  if (!isRecord(value))
    throw new Error(`${label}格式无效`)
  return value
}

export function stringArray(value: unknown, label: string): string[] {
  if (!Array.isArray(value) || value.some(item => typeof item !== 'string'))
    throw new Error(`${label}格式无效`)
  return [...value]
}
