<script setup lang="ts">
import type { Evidence } from '../api'
import type { ExampleGuide, ExampleRun } from '../types'
import { Check, CircleAlert, LoaderCircle } from '@lucide/vue'
import { computed } from 'vue'
import { requestMilestones } from '../constants/examples'

const props = defineProps<{ example: ExampleGuide, run?: ExampleRun, evidence: Evidence[] }>()
const running = computed(() => props.run?.phase === 'running')
const account = computed(() => props.evidence.find(item => item.capability === 'scheduler' && item.accountId)?.accountId)
const milestones = computed(() => requestMilestones.filter(item => props.evidence.some(event => event.event === item.event)))
const status = computed(() => {
  if (!props.run)
    return ''
  return { running: '正在处理', done: '已返回', stopped: '已停止', error: '请求失败' }[props.run.phase]
})
</script>

<template>
  <div class="grid gap-4 rounded-cp bg-cp-fill-quaternary p-4" aria-live="polite">
    <div class="flex items-center justify-between gap-3 text-cp-sm">
      <span class="font-emphasis">{{ example.action === 'uppercase' ? '转换结果' : '返回结果' }}</span>
      <span v-if="run" class="inline-flex items-center gap-1.5 text-cp-xs text-cp-text-tertiary">
        <LoaderCircle v-if="running" class="size-3.5 animate-spin motion-reduce:animate-none" />
        <Check v-else-if="run.phase === 'done'" class="size-3.5 text-cp-success-text" />
        {{ status }}
      </span>
    </div>
    <p v-if="run?.error" class="m-0 flex items-start gap-2 text-cp-sm text-cp-error-text" role="alert">
      <CircleAlert class="mt-0.5 size-4 shrink-0" />{{ run.error }}
    </p>
    <template v-if="run?.output">
      <div v-if="example.group === 'interactive'" class="grid gap-3 sm:grid-cols-2">
        <div class="min-w-0 rounded-cp bg-cp-bg-container p-3">
          <span class="text-cp-xs text-cp-text-tertiary">本次输入</span>
          <p class="mb-0 mt-2 whitespace-pre-wrap break-words font-mono text-cp-sm">
            {{ run.input }}
          </p>
        </div>
        <div class="min-w-0 rounded-cp bg-cp-bg-container p-3">
          <span class="text-cp-xs text-cp-text-tertiary">{{ example.action === 'echo' ? '插件返回' : '模型回复' }}</span>
          <p class="mb-0 mt-2 whitespace-pre-wrap break-words font-mono text-cp-sm">
            {{ run.output }}
          </p>
        </div>
      </div>
    </template>
    <p v-else-if="!run?.error" class="m-0 text-cp-sm leading-relaxed text-cp-text-secondary">
      {{ running ? '正在等待插件返回…' : example.expected }}
    </p>

    <template v-if="example.action === 'request' && run">
      <dl class="m-0 grid grid-cols-2 gap-4 lg:grid-cols-4">
        <div>
          <dt class="text-cp-xs text-cp-text-tertiary">
            请求模型
          </dt><dd class="m-0 mt-1 font-mono text-cp-sm">
            {{ run.requestedModel }}
          </dd>
        </div>
        <div>
          <dt class="text-cp-xs text-cp-text-tertiary">
            实际模型
          </dt><dd class="m-0 mt-1 font-mono text-cp-sm">
            {{ run.model ?? '等待返回' }}
          </dd>
        </div>
        <div>
          <dt class="text-cp-xs text-cp-text-tertiary">
            执行账号
          </dt><dd class="m-0 mt-1 break-all text-cp-sm">
            {{ account ?? '等待记录' }}
          </dd>
        </div>
        <div>
          <dt class="text-cp-xs text-cp-text-tertiary">
            SSE 文本分块
          </dt><dd class="m-0 mt-1 font-mono text-cp-sm">
            {{ run.chunks }}
          </dd>
        </div>
      </dl>
      <dl v-if="run.totalTokens !== null" class="m-0 flex flex-wrap gap-x-6 gap-y-2 text-cp-xs text-cp-text-secondary">
        <div class="flex gap-2">
          <dt>输入 Token</dt><dd class="m-0 font-mono">
            {{ run.inputTokens ?? '—' }}
          </dd>
        </div>
        <div class="flex gap-2">
          <dt>输出 Token</dt><dd class="m-0 font-mono">
            {{ run.outputTokens ?? '—' }}
          </dd>
        </div>
        <div class="flex gap-2">
          <dt>总 Token</dt><dd class="m-0 font-mono">
            {{ run.totalTokens }}
          </dd>
        </div>
      </dl>
      <div v-if="milestones.length" class="flex flex-wrap gap-x-4 gap-y-2 text-cp-xs text-cp-text-secondary">
        <span v-for="item in milestones" :key="item.event" class="inline-flex items-center gap-1.5">
          <Check class="size-3.5 text-cp-success-text" />{{ item.label }}
        </span>
      </div>
      <p class="m-0 text-cp-xs text-cp-text-tertiary">
        这里演示 HTTP / SSE，WebSocket 帧观察需使用 WebSocket 客户端触发
      </p>
    </template>
    <div v-if="run && (run.requestId || run.durationMs !== null)" class="flex flex-wrap justify-between gap-2 text-cp-xs text-cp-text-tertiary">
      <code v-if="run.requestId" class="break-all">请求 ID · {{ run.requestId }}</code>
      <span v-if="run.durationMs !== null">总耗时 {{ run.durationMs }} ms</span>
    </div>
  </div>
</template>
