<script setup lang="ts">
import type { WorkbenchView } from '../types'
import { BaseSegmented } from '@codex-proxy/ui'
import { workbenchViews } from '../constants/workbench'

const model = defineModel<WorkbenchView>({ required: true })
</script>

<template>
  <nav class="flex shrink-0 flex-wrap items-center justify-between gap-3 pr-2" aria-label="工作台视图">
    <BaseSegmented v-model="model" label="工作台视图" :options="workbenchViews" />
    <span v-if="model === 'text'" class="text-cp-xs text-cp-text-tertiary">
      摘要、翻译与改写
    </span>
  </nav>
</template>
