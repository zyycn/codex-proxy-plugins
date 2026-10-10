<script setup lang="ts">
import type { UiSelectOption } from '../../types'
import { ZButton, ZPopover, ZSelect } from '@codex-proxy/ui'
import { Plus, SlidersHorizontal } from '@lucide/vue'

const props = defineProps<{
  keyOptions: UiSelectOption[]
  modelOptions: UiSelectOption[]
  disabled: boolean
  locked: boolean
  loading: boolean
}>()
defineEmits<{ new: [] }>()
const clientKeyId = defineModel<string>('clientKeyId', { required: true })
const modelId = defineModel<string>('modelId', { required: true })
const quality = defineModel<string>('quality', { required: true })
const size = defineModel<string>('size', { required: true })
const qualities = [{ label: '低', value: 'low' }, { label: '中', value: 'medium' }, { label: '高', value: 'high' }]
const sizes = [
  { label: '正方形 · 1024 × 1024', value: '1024x1024' },
  { label: '横图 · 1536 × 1024', value: '1536x1024' },
  { label: '竖图 · 1024 × 1536', value: '1024x1536' },
]
</script>

<template>
  <header class="flex shrink-0 flex-wrap items-end gap-3 border-b border-cp-border-secondary p-4">
    <div class="grid min-w-0 flex-1 basis-40 gap-1.5">
      <span class="text-cp-xs text-cp-text-secondary">客户端 Key</span>
      <ZSelect v-model="clientKeyId" :options="keyOptions" :disabled="disabled || locked || !keyOptions.length" placeholder="暂无可用 Key" aria-label="图片客户端 Key" />
    </div>
    <div class="grid min-w-0 flex-1 basis-40 gap-1.5">
      <span class="text-cp-xs text-cp-text-secondary">模型</span>
      <ZSelect v-model="modelId" :options="modelOptions" :disabled="disabled || locked || loading || !modelOptions.length" :placeholder="loading ? '正在读取模型' : '暂无可用模型'" aria-label="图片模型" />
    </div>
    <div class="flex items-center gap-2">
      <ZPopover placement="bottom-end" :disabled="props.disabled">
        <template #reference="{ open }">
          <ZButton :disabled="disabled" :aria-expanded="open" aria-label="图片设置">
            <template #icon>
              <SlidersHorizontal class="size-4" />
            </template>
            图片设置
          </ZButton>
        </template>
        <div class="grid w-64 max-w-[calc(100vw-2rem)] gap-4 p-4">
          <div class="grid gap-1.5">
            <span class="text-cp-xs text-cp-text-secondary">画质</span>
            <ZSelect v-model="quality" :options="qualities" :disabled="disabled" aria-label="画质" />
          </div>
          <div class="grid gap-1.5">
            <span class="text-cp-xs text-cp-text-secondary">尺寸</span>
            <ZSelect v-model="size" :options="sizes" :disabled="disabled" aria-label="图片尺寸" />
          </div>
          <p class="m-0 text-cp-xs leading-relaxed text-cp-text-tertiary">
            按所选 Key 计费，会话中固定 Key 与模型，刷新页面会清空当前对话
          </p>
        </div>
      </ZPopover>
      <ZButton :disabled="disabled" @click="$emit('new')">
        <template #icon>
          <Plus class="size-4" />
        </template>
        新对话
      </ZButton>
    </div>
  </header>
</template>
