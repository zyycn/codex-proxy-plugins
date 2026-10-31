<script setup lang="ts">
import type { Evidence } from '../api'
import { ZScrollbar } from '@codex-proxy/ui'
import { ChevronDown } from '@lucide/vue'
import { formatDateTime } from '../utils/workbench'

defineProps<{ evidence: Evidence[], capabilities: string[] }>()
</script>

<template>
  <details class="group rounded-z bg-z-fill-quaternary">
    <summary class="flex cursor-pointer list-none items-center justify-between gap-3 rounded-z px-4 py-3 outline-none focus-visible:ring-2 focus-visible:ring-z-control-outline [&::-webkit-details-marker]:hidden">
      <span class="text-z-sm text-z-text-secondary">开发说明与原始记录</span>
      <ChevronDown class="size-4 text-z-text-tertiary transition-transform group-open:rotate-180 motion-reduce:transition-none" />
    </summary>
    <div class="grid gap-3 px-4 pb-4">
      <p class="m-0 text-z-xs text-z-text-tertiary">
        在线示例显示本次调用，接入指南显示当前进程的最近记录
      </p>
      <div v-for="capability in capabilities" :key="capability" class="flex flex-wrap items-center justify-between gap-2 text-z-xs">
        <code class="font-mono text-z-text">{{ capability }}</code>
        <span class="text-z-text-tertiary">{{ evidence.some(item => item.capability === capability) ? '有调用记录' : '相关扩展能力' }}</span>
      </div>
      <article v-for="item in evidence" :key="item.id" class="rounded-z bg-z-bg-container p-3">
        <div class="flex flex-wrap items-center justify-between gap-2 text-z-xs">
          <code class="font-mono text-z-text">{{ item.event }}</code>
          <time class="text-z-text-tertiary">{{ formatDateTime(item.occurredAtMs) }}</time>
        </div>
        <ZScrollbar v-if="Object.keys(item.details).length" max-height="12rem" horizontal class="mt-2">
          <pre class="m-0 p-1 font-mono text-z-xs leading-relaxed text-z-text-secondary">{{ JSON.stringify(item.details, null, 2) }}</pre>
        </ZScrollbar>
      </article>
    </div>
  </details>
</template>
