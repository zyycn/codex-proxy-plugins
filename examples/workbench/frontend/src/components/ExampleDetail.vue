<script setup lang="ts">
import type { Evidence } from '../api'
import type { ExampleGuide, ExampleRun, UiSelectOption } from '../types'
import { BaseButton, BaseInput, BaseMarkdown, BaseSelect } from '@codex-proxy/ui'
import { Square } from '@lucide/vue'
import { computed } from 'vue'
import ExampleEvidence from './ExampleEvidence.vue'
import ExampleResult from './ExampleResult.vue'

const props = defineProps<{
  example: ExampleGuide
  evidence: Evidence[]
  keyOptions: UiSelectOption[]
  run?: ExampleRun
  busy: boolean
}>()
const emit = defineEmits<{ run: [], stop: [] }>()
defineSlots<{ actions?: () => unknown }>()
const demoKeyId = defineModel<string>('demoKeyId', { required: true })
const message = defineModel<string>('message', { required: true })
const running = computed(() => props.run?.phase === 'running')
const usesModel = computed(() => ['uppercase', 'request'].includes(props.example.action))
const commandMarkdown = computed(() => props.example.command ? `\`\`\`sh\n${props.example.command}\n\`\`\`` : '')
const disabled = computed(() => props.busy
  || (usesModel.value && !demoKeyId.value)
  || (props.example.group === 'interactive' && !message.value.trim()))
</script>

<template>
  <section class="grid min-w-0 content-start gap-5 p-4 pt-0 sm:px-6 sm:pb-6 lg:pt-6" :aria-label="example.title">
    <header class="flex items-start justify-between gap-4">
      <div class="grid min-w-0 gap-1.5">
        <h2 class="m-0 text-cp-lg font-emphasis">
          {{ example.title }}
        </h2>
        <p class="m-0 max-w-3xl text-cp-sm leading-relaxed text-cp-text-secondary">
          {{ example.summary }}
        </p>
      </div>
      <div class="shrink-0">
        <slot name="actions" />
      </div>
    </header>

    <template v-if="example.group === 'interactive'">
      <div class="grid items-end gap-3 sm:grid-cols-[minmax(10rem,16rem)_1fr_auto]">
        <div v-if="usesModel" class="grid gap-2 text-cp-sm text-cp-text-secondary">
          客户端 Key
          <BaseSelect v-model="demoKeyId" :options="keyOptions" :disabled="busy" placeholder="请选择可用 Key" aria-label="示例客户端 Key" />
        </div>
        <div class="grid gap-2 text-cp-sm text-cp-text-secondary" :class="usesModel ? '' : 'sm:col-span-2'">
          {{ example.action === 'uppercase' ? '转换前的文字' : '发送内容' }}
          <BaseInput v-model="message" :disabled="busy" maxlength="4096" aria-label="示例输入文本" @keydown.enter.prevent="!disabled && emit('run')" />
        </div>
        <BaseButton v-if="running" @click="emit('stop')">
          <template #icon>
            <Square class="size-3.5" />
          </template>停止
        </BaseButton>
        <BaseButton v-else variant="primary" :disabled="disabled" @click="emit('run')">
          {{ example.actionLabel }}
        </BaseButton>
      </div>
      <p v-if="usesModel" class="m-0 text-cp-xs text-cp-text-tertiary">
        在本地运行，会复用或创建两个演示账号，并经过所选 Key 的计量，不调用外部模型
      </p>
      <p v-if="usesModel" class="m-0 text-cp-xs text-cp-text-secondary">
        先在插件管理的当前配置 → 生效请求中开启{{ example.action === 'uppercase' ? '“请求中间件 · 请求开始”' : '模型路由、账号调度与请求观察' }}，并将所选 Key 纳入生效范围。
      </p>
      <ExampleResult :example="example" :run="run" :evidence="evidence" />
    </template>

    <template v-else>
      <ol class="m-0 grid list-decimal gap-3 pl-5 text-cp-sm leading-relaxed text-cp-text-secondary marker:text-cp-text-tertiary">
        <li v-for="step in example.steps" :key="step" class="pl-1">
          {{ step }}
        </li>
      </ol>
      <BaseMarkdown v-if="commandMarkdown" :source="commandMarkdown" />
      <div v-if="example.action === 'accounts'" class="flex">
        <BaseButton variant="primary" :loading="running" :disabled="busy && !running" @click="emit('run')">
          {{ example.actionLabel }}
        </BaseButton>
      </div>
      <p class="m-0 text-cp-sm text-cp-text-secondary">
        {{ example.expected }}
      </p>
      <ExampleResult v-if="run" :example="example" :run="run" :evidence="evidence" />
    </template>

    <ExampleEvidence :evidence="evidence" :capabilities="example.capabilities" />
  </section>
</template>
