<script setup lang="ts">
import type { UiSelectOption } from '../types'
import { BaseIconButton, BaseSelect } from '@codex-proxy/ui'
import { FilePlus2, RefreshCw } from '@lucide/vue'

defineProps<{
  keyOptions: UiSelectOption[]
  modelOptions: UiSelectOption[]
  historyOptions: UiSelectOption[]
  modelsLoading: boolean
  historyLoading: boolean
  disabled: boolean
}>()

const emit = defineEmits<{
  refreshHistory: []
  newTask: []
}>()

const clientKeyId = defineModel<string>('clientKeyId', { required: true })
const modelId = defineModel<string>('modelId', { required: true })
const historyId = defineModel<string>('historyId', { required: true })
</script>

<template>
  <div class="grid min-w-0 gap-3 md:grid-cols-[minmax(10rem,0.8fr)_minmax(11rem,1fr)_minmax(11rem,1fr)_auto]">
    <div class="grid min-w-0 gap-1.5 text-cp-xs font-emphasis text-cp-text-secondary">
      <span>客户端 Key</span>
      <BaseSelect
        v-model="clientKeyId"
        :options="keyOptions"
        :disabled="disabled || keyOptions.length === 0"
        placeholder="暂无可用 Key"
        empty-text="暂无可用 Key"
        aria-label="客户端 Key"
      />
    </div>
    <div class="grid min-w-0 gap-1.5 text-cp-xs font-emphasis text-cp-text-secondary">
      <span>模型</span>
      <BaseSelect
        v-model="modelId"
        :options="modelOptions"
        :disabled="disabled || modelsLoading || modelOptions.length === 0"
        :placeholder="modelsLoading ? '正在读取模型' : '暂无可用真实模型'"
        empty-text="暂无可用真实模型"
        aria-label="模型"
      />
    </div>
    <div class="grid min-w-0 gap-1.5 text-cp-xs font-emphasis text-cp-text-secondary">
      <span>历史记录</span>
      <BaseSelect
        v-model="historyId"
        :options="historyOptions"
        :disabled="disabled || historyLoading"
        placeholder="新任务"
        empty-text="暂无保存记录"
        aria-label="历史记录"
      />
    </div>
    <div class="flex items-end gap-1">
      <BaseIconButton
        label="重新载入历史记录"
        :loading="historyLoading"
        :disabled="disabled"
        @click="emit('refreshHistory')"
      >
        <RefreshCw />
      </BaseIconButton>
      <BaseIconButton label="新建任务" :disabled="disabled" @click="emit('newTask')">
        <FilePlus2 />
      </BaseIconButton>
    </div>
  </div>
</template>
