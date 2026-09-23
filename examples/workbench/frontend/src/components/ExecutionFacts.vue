<script setup lang="ts">
import type { ResponseFacts } from '../types'
import { BaseScrollbar } from '@codex-proxy/ui'
import { Activity, ChevronDown } from '@lucide/vue'
import { computed } from 'vue'
import { formatDateTime } from '../utils/workbench'

const props = defineProps<{
  facts?: ResponseFacts
}>()

const rows = computed(() => {
  const facts = props.facts
  if (!facts)
    return []
  return [
    ['请求 ID', facts.requestId],
    ['响应 ID', facts.responseId],
    ['实际模型', facts.model],
    ['输入 Token', facts.inputTokens],
    ['输出 Token', facts.outputTokens],
    ['总 Token', facts.totalTokens],
    ['页面等待', facts.durationMs === null ? null : `${facts.durationMs} ms`],
  ] as Array<[string, string | number | null]>
})
</script>

<template>
  <details class="group rounded-cp-card bg-(--cp-card-bg) shadow-cp-card">
    <summary class="flex min-h-12 cursor-pointer list-none items-center justify-between gap-3 rounded-cp-card px-4 py-3 outline-none focus-visible:ring-2 focus-visible:ring-cp-control-outline [&::-webkit-details-marker]:hidden">
      <span class="inline-flex min-w-0 items-center gap-2 font-emphasis text-cp-text">
        <Activity class="size-4 shrink-0 text-cp-primary-text" aria-hidden="true" />
        请求详情
        <span class="text-cp-xs font-normal text-cp-text-tertiary">
          {{ facts ? [facts.model, facts.totalTokens == null ? null : `${facts.totalTokens} Token`, facts.durationMs == null ? null : `${facts.durationMs} ms`].filter(Boolean).join(' · ') : '生成后查看模型与用量' }}
        </span>
      </span>
      <ChevronDown class="size-4 shrink-0 text-cp-text-tertiary transition-transform duration-150 group-open:rotate-180 motion-reduce:transition-none" aria-hidden="true" />
    </summary>

    <div class="grid gap-4 px-4 pb-4">
      <dl v-if="facts" class="m-0 grid gap-x-6 gap-y-2 text-cp-sm sm:grid-cols-2 lg:grid-cols-3">
        <div v-for="([label, value]) in rows" :key="label" class="min-w-0">
          <dt class="text-cp-text-tertiary">
            {{ label }}
          </dt>
          <dd class="mt-0.5 mb-0 wrap-anywhere text-cp-text">
            {{ value ?? '宿主未提供' }}
          </dd>
        </div>
      </dl>
      <p v-else class="m-0 text-cp-sm text-cp-text-secondary">
        完成一次生成后，可查看请求 ID、实际模型与 Token 用量
      </p>

      <BaseScrollbar v-if="facts?.evidence.length" max-height="16rem">
        <div class="grid gap-1 pr-3">
          <article
            v-for="item in facts.evidence"
            :key="item.id"
            class="rounded-cp px-3 py-2 odd:bg-cp-fill-quaternary"
          >
            <div class="flex flex-wrap items-center justify-between gap-2">
              <span class="font-mono text-cp-xs text-cp-text">{{ item.capability }} · {{ item.event }}</span>
              <time class="text-cp-xs text-cp-text-tertiary" :datetime="new Date(item.occurredAtMs).toISOString()">
                {{ formatDateTime(item.occurredAtMs) }}
              </time>
            </div>
            <p v-if="item.provider || item.model || item.accountId" class="mt-1.5 mb-0 text-cp-xs text-cp-text-secondary">
              {{ [item.provider, item.model, item.accountId].filter(Boolean).join(' · ') }}
            </p>
          </article>
        </div>
      </BaseScrollbar>
    </div>
  </details>
</template>
