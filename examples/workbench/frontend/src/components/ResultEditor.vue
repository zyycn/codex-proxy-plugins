<script setup lang="ts">
import type { GenerationPhase } from '../types'
import { BaseButton, BaseIconButton, BaseInput, BaseMarkdown, BaseScrollbar, BaseTextarea, toast } from '@codex-proxy/ui'
import { AlignLeft, ArrowUp, Check, CircleAlert, Copy, LoaderCircle, Pencil, Save, Send, Square } from '@lucide/vue'
import { computed, shallowRef, watch } from 'vue'
import { generationLabels } from '../constants/workbench'

const props = defineProps<{
  phase: GenerationPhase
  running: boolean
  loading: boolean
  canGenerate: boolean
  saving: boolean
}>()
const emit = defineEmits<{
  generate: []
  retry: []
  stop: []
  save: []
  continue: [instruction: string]
}>()
const result = defineModel<string>({ required: true })
const adjustment = shallowRef('')
const copied = shallowRef(false)
const editing = shallowRef(false)
watch(result, () => {
  copied.value = false
})
const statusLabel = computed(() => generationLabels[props.phase])

async function copyResult() {
  try {
    await navigator.clipboard.writeText(result.value)
    copied.value = true
  }
  catch {
    toast.error('复制失败，可切换编辑模式后手动复制')
  }
}

function continueWith(value: string) {
  if (!value.trim() || !props.canGenerate)
    return
  emit('continue', value.trim())
  adjustment.value = ''
  editing.value = false
  copied.value = false
}
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-col p-4 sm:p-5" aria-labelledby="result-title">
    <div class="flex items-center justify-between gap-3">
      <div>
        <h2 id="result-title" class="m-0 text-cp font-emphasis">
          结果
        </h2>
        <p class="mt-1 mb-0 flex items-center gap-1.5 text-cp-xs text-cp-text-tertiary" role="status">
          <LoaderCircle v-if="running" class="size-3 animate-spin motion-reduce:animate-none" />
          {{ statusLabel }}
        </p>
      </div>
      <div class="flex gap-1">
        <BaseIconButton :label="editing ? '预览结果' : '编辑结果'" :pressed="editing" :disabled="loading || !result || running" @click="editing = !editing">
          <AlignLeft v-if="editing" /><Pencil v-else />
        </BaseIconButton>
        <BaseIconButton :label="copied ? '已复制' : '复制结果'" :disabled="loading || !result || running" @click="copyResult">
          <Check v-if="copied" /><Copy v-else />
        </BaseIconButton>
        <BaseIconButton label="保存记录" :loading="saving" :disabled="loading || !result || running" @click="emit('save')">
          <Save />
        </BaseIconButton>
      </div>
    </div>

    <div class="mt-4 h-64 min-w-0 sm:h-80">
      <BaseTextarea
        v-if="editing && !running"
        v-model="result"
        class="h-full [&_textarea]:h-full [&_textarea]:font-normal"
        resize="none"
        aria-label="编辑生成结果"
      />
      <BaseScrollbar v-else-if="result" class="h-full" aria-label="生成结果">
        <BaseMarkdown :source="result" class="px-1 pr-4" />
      </BaseScrollbar>
      <div v-else class="flex h-full flex-col items-center justify-center gap-3 rounded-cp bg-cp-fill-quaternary text-cp-text-tertiary">
        <CircleAlert v-if="phase === 'error'" class="size-7" aria-hidden="true" />
        <AlignLeft v-else class="size-7" :class="running ? 'animate-pulse motion-reduce:animate-none' : ''" aria-hidden="true" />
        <span class="text-cp-sm">{{ loading ? '正在读取工作台' : running ? '正在生成…' : phase === 'error' ? '生成失败，可重新生成' : '填写原文，选择一种处理方式' }}</span>
      </div>
    </div>

    <div v-if="!loading && result && !running" class="mt-3 flex flex-wrap items-center gap-2">
      <BaseButton size="sm" variant="soft" :disabled="!canGenerate" @click="continueWith('在保持事实准确的前提下，把当前结果压缩约三分之一')">
        更简短
      </BaseButton>
      <BaseButton size="sm" variant="soft" :disabled="!canGenerate" @click="continueWith('保持原意，把当前结果改成更自然、更有亲和力的语气')">
        更自然
      </BaseButton>
      <div class="flex min-w-40 flex-1 items-center gap-1">
        <BaseInput v-model="adjustment" class="flex-1" maxlength="512" placeholder="继续调整…" aria-label="继续调整要求" @keydown.enter.prevent="continueWith(adjustment)" />
        <BaseIconButton label="发送调整要求" variant="filled" :disabled="!adjustment.trim() || !canGenerate" @click="continueWith(adjustment)">
          <ArrowUp />
        </BaseIconButton>
      </div>
    </div>

    <div class="mt-auto flex flex-wrap items-center justify-between gap-3 pt-4">
      <span class="text-cp-xs text-cp-text-tertiary">真实模型调用，按所选 Key 计费</span>
      <BaseButton v-if="running" @click="emit('stop')">
        <template #icon>
          <Square class="size-3.5" />
        </template>停止生成
      </BaseButton>
      <BaseButton v-else variant="primary" :disabled="!canGenerate" @click="phase === 'error' ? emit('retry') : emit('generate')">
        <template #icon>
          <Send class="size-4" />
        </template>{{ phase === 'error' ? '重新生成' : '生成' }}
      </BaseButton>
    </div>
  </section>
</template>
