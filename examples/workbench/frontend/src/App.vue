<script setup lang="ts">
import type { WorkbenchView } from './types'
import { BaseToast } from '@codex-proxy/ui'
import { FlaskConical } from '@lucide/vue'
import { shallowRef } from 'vue'
import WorkbenchNavigation from './components/WorkbenchNavigation.vue'
import { useWorkbenchData } from './composables/useWorkbenchData'
import BasicExamples from './views/examples/index.vue'
import TextWorkbench from './views/text-workbench/index.vue'

defineProps<{
  preview: boolean
}>()

const view = shallowRef<WorkbenchView>('text')
const { snapshot, loading, error, refresh } = useWorkbenchData()
</script>

<template>
  <main
    class="flex min-w-0 flex-col gap-4 font-sans text-cp-text"
    :style="{ minHeight: preview ? '100dvh' : 'inherit' }"
  >
    <div
      v-if="preview"
      class="flex flex-wrap items-center justify-between gap-2 rounded-cp bg-cp-warning-container px-3 py-2 text-cp-sm text-cp-warning-on-container"
      role="note"
    >
      <span class="inline-flex items-center gap-2">
        <FlaskConical class="size-4 shrink-0" aria-hidden="true" />
        独立模拟预览，不代表宿主安装或集成结果
      </span>
    </div>

    <WorkbenchNavigation v-model="view" />
    <TextWorkbench
      v-show="view === 'text'"
      :snapshot="snapshot"
      :loading="loading"
      :error="error"
      :refresh="refresh"
    />
    <BasicExamples
      v-show="view === 'examples'"
      :snapshot="snapshot"
      :loading="loading"
      :error="error"
      :refresh="refresh"
    />
  </main>
  <BaseToast />
</template>
