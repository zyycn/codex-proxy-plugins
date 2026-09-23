<script setup lang="ts">
import type { FetchTextReply, SourceKind } from '../api'
import { BaseButton, BaseInput, BaseSegmented, BaseTextarea } from '@codex-proxy/ui'
import { Download } from '@lucide/vue'
import { computed } from 'vue'
import { sourceKindsOptions } from '../constants/workbench'
import { formatBytes } from '../utils/workbench'

const props = defineProps<{
  disabled: boolean
  fetching: boolean
  fetchReply?: FetchTextReply
}>()

const emit = defineEmits<{
  fetch: []
}>()

const source = defineModel<string>({ required: true })
const kind = defineModel<SourceKind>('kind', { required: true })
const url = defineModel<string>('url', { required: true })

const sourceCount = computed(() => source.value.length.toLocaleString('zh-CN'))
const fetchSummary = computed(() => {
  if (!props.fetchReply)
    return ''
  const suffix = props.fetchReply.truncated ? ' · 已截断' : ''
  return `HTTP ${props.fetchReply.status} · ${formatBytes(props.fetchReply.bytes)}${suffix}`
})
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-col p-4 sm:p-5" aria-labelledby="source-title">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 id="source-title" class="m-0 text-cp font-emphasis text-cp-text">
          原文
        </h2>
        <p class="mt-1 mb-0 text-cp-xs text-cp-text-tertiary">
          {{ sourceCount }} 字符
        </p>
      </div>
      <BaseSegmented v-model="kind" label="原文来源" :options="sourceKindsOptions" size="sm" :disabled="disabled" />
    </div>

    <div v-if="kind === 'url'" class="mt-4 grid gap-2 sm:grid-cols-[1fr_auto]">
      <BaseInput
        v-model="url"
        type="url"
        :disabled="disabled || fetching"
        placeholder="https://example.com/article"
        autocomplete="url"
        aria-label="网页地址"
        @keydown.enter.prevent="emit('fetch')"
      />
      <BaseButton :loading="fetching" :disabled="disabled || !url.trim()" @click="emit('fetch')">
        <template #icon>
          <Download class="size-4" />
        </template>
        取文
      </BaseButton>
      <p v-if="fetchSummary" class="m-0 text-cp-xs text-cp-text-tertiary sm:col-span-2" role="status">
        {{ fetchSummary }}
      </p>
    </div>

    <BaseTextarea
      v-model="source"
      class="mt-4 [&_textarea]:h-64 [&_textarea]:font-normal [&_textarea]:leading-relaxed sm:[&_textarea]:h-80"
      :disabled="disabled"
      :rows="10"
      resize="none"
      maxlength="262144"
      placeholder="粘贴文章、会议记录或需要改写的段落"
      aria-label="待处理原文"
    />
    <div class="mt-3 flex items-center justify-between gap-2">
      <span class="text-cp-xs text-cp-text-tertiary">保留原文，生成后可继续调整结果</span>
      <BaseButton :class="{ invisible: Boolean(source) }" size="sm" :disabled="disabled" @click="source = 'Our team released a text workbench. It can summarize, translate, and rewrite articles. Users can stop generation and save their results.'">
        填入示例
      </BaseButton>
    </div>
  </section>
</template>
