<script setup lang="ts">
import type { ResponseFacts } from '../types'
import { ZScrollbar } from '@codex-proxy/ui'
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
  <details class="group rounded-z-card bg-(--z-card-bg) shadow-z-card">
    <summary class="flex min-h-12 cursor-pointer list-none items-center justify-between gap-3 rounded-z-card px-4 py-3 outline-none focus-visible:ring-2 focus-visible:ring-z-control-outline [&::-webkit-details-marker]:hidden">
      <span class="inline-flex min-w-0 items-center gap-2 font-emphasis text-z-text">
        <Activity class="size-4 shrink-0 text-z-primary-text" aria-hidden="true" />
        请求详情
        <span class="text-z-xs font-normal text-z-text-tertiary">
          {{ facts ? [facts.model, facts.totalTokens == null ? null : `${facts.totalTokens} Token`, facts.durationMs == null ? null : `${facts.durationMs} ms`].filter(Boolean).join(' · ') : '生成后查看模型与用量' }}
        </span>
      </span>
      <ChevronDown class="size-4 shrink-0 text-z-text-tertiary transition-transform duration-150 group-open:rotate-180 motion-reduce:transition-none" aria-hidden="true" />
    </summary>

    <div class="grid gap-4 px-4 pb-4">
      <dl v-if="facts" class="m-0 grid gap-x-6 gap-y-2 text-z-sm sm:grid-cols-2 lg:grid-cols-3">
        <div v-for="([label, value]) in rows" :key="label" class="min-w-0">
          <dt class="text-z-text-tertiary">
            {{ label }}
          </dt>
          <dd class="mt-0.5 mb-0 wrap-anywhere text-z-text">
            {{ value ?? '宿主未提供' }}
          </dd>
        </div>
      </dl>
      <p v-else class="m-0 text-z-sm text-z-text-secondary">
        完成一次生成后，可查看请求 ID、实际模型与 Token 用量
      </p>

      <ZScrollbar v-if="facts?.evidence.length" max-height="16rem">
        <div class="grid gap-1 pr-3">
          <article
            v-for="item in facts.evidence"
            :key="item.id"
            class="rounded-z px-3 py-2 odd:bg-z-fill-quaternary"
          >
            <div class="flex flex-wrap items-center justify-between gap-2">
              <span class="font-mono text-z-xs text-z-text">{{ item.capability }} · {{ item.event }}</span>
              <time class="text-z-xs text-z-text-tertiary" :datetime="new Date(item.occurredAtMs).toISOString()">
                {{ formatDateTime(item.occurredAtMs) }}
              </time>
            </div>
            <p v-if="item.provider || item.model || item.accountId" class="mt-1.5 mb-0 text-z-xs text-z-text-secondary">
              {{ [item.provider, item.model, item.accountId].filter(Boolean).join(' · ') }}
            </p>
          </article>
        </div>
      </ZScrollbar>
    </div>
  </details>
</template>
