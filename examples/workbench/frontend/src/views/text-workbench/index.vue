<script setup lang="ts">
import type { WorkbenchSnapshot } from '../../api'
import type { UiSegmentedOption } from '../../types'
import { BaseCard, BaseInput, BaseSegmented } from '@codex-proxy/ui'
import { CircleAlert } from '@lucide/vue'
import { computed, toRef } from 'vue'
import ExecutionFacts from '../../components/ExecutionFacts.vue'
import ResourceToolbar from '../../components/ResourceToolbar.vue'
import ResultEditor from '../../components/ResultEditor.vue'
import SourceEditor from '../../components/SourceEditor.vue'
import { useTextWorkbench } from '../../composables/useTextWorkbench'
import { taskOptions } from '../../constants/workbench'

const props = defineProps<{
  snapshot?: WorkbenchSnapshot
  loading: boolean
  error: string
  refresh: () => Promise<void>
}>()

const workbench = useTextWorkbench({
  snapshot: toRef(props, 'snapshot'),
  refreshSnapshot: props.refresh,
})

const instructionLabel = computed(() => ({
  summarize: '摘要要求',
  translate: '目标语言与要求',
  rewrite: '改写语气与要求',
}[workbench.task.value]))

const taskSegmentOptions = taskOptions as UiSegmentedOption[]
const initializing = computed(() => (props.loading && !props.snapshot) || workbench.historyLoading.value)
const editingDisabled = computed(() => initializing.value || workbench.running.value)
const resourceError = computed(() => [
  props.error,
  workbench.modelsError.value,
  workbench.historyError.value,
].find(Boolean) ?? '')
const noRealModels = computed(() => Boolean(
  workbench.clientKeyId.value
  && !workbench.modelsLoading.value
  && !workbench.modelsError.value
  && workbench.modelOptions.value.length === 0,
))
</script>

<template>
  <div class="grid min-w-0 gap-4" :aria-busy="initializing">
    <BaseCard padding="compact">
      <ResourceToolbar
        v-model:client-key-id="workbench.clientKeyId.value"
        v-model:model-id="workbench.modelId.value"
        v-model:history-id="workbench.selectedTaskId.value"
        :key-options="workbench.keyOptions.value"
        :model-options="workbench.modelOptions.value"
        :history-options="workbench.historyOptions.value"
        :models-loading="workbench.modelsLoading.value"
        :history-loading="workbench.historyLoading.value"
        :disabled="editingDisabled"
        @refresh-history="workbench.loadHistory"
        @new-task="workbench.newTask"
      />

      <div class="mt-4 grid gap-3 border-0 pt-0 md:grid-cols-[auto_minmax(12rem,1fr)] md:items-end">
        <div class="grid gap-1.5 text-cp-xs font-emphasis text-cp-text-secondary">
          <span>文本任务</span>
          <BaseSegmented
            v-model="workbench.task.value"
            label="文本任务"
            :options="taskSegmentOptions"
            :disabled="editingDisabled"
          />
        </div>
        <div class="grid min-w-0 gap-1.5 text-cp-xs font-emphasis text-cp-text-secondary">
          <span>{{ instructionLabel }}</span>
          <BaseInput
            v-model="workbench.instruction.value"
            :disabled="editingDisabled"
            maxlength="512"
            :placeholder="workbench.defaultInstruction.value"
            :aria-label="instructionLabel"
          />
        </div>
      </div>

      <p v-if="resourceError" class="mt-3 mb-0 flex items-start gap-2 text-cp-sm text-cp-error-text" role="alert">
        <CircleAlert class="mt-0.5 size-4 shrink-0" aria-hidden="true" />
        {{ resourceError }}
      </p>
      <p v-else-if="workbench.historyNotice.value" class="mt-3 mb-0 text-cp-xs text-cp-success-text" role="status">
        {{ workbench.historyNotice.value }}
      </p>
      <p
        v-else-if="snapshot?.keysNextCursor"
        class="mt-3 mb-0 text-cp-xs text-cp-text-tertiary"
      >
        Key 列表还有更多结果，本示例只展示首批候选
      </p>
      <p v-else-if="noRealModels" class="mt-3 mb-0 text-cp-xs text-cp-text-tertiary">
        当前 Key 没有可用的真实模型，可以切换 Key 或先在账号管理中添加模型账号
      </p>
    </BaseCard>

    <BaseCard padding="none" class="min-w-0 overflow-hidden">
      <div class="grid min-w-0 lg:grid-cols-2">
        <SourceEditor
          v-model="workbench.source.value"
          v-model:kind="workbench.sourceKind.value"
          v-model:url="workbench.sourceUrl.value"
          class="bg-cp-bg-container"
          :disabled="editingDisabled"
          :fetching="workbench.fetchingText.value"
          :fetch-reply="workbench.fetchReply.value"
          @fetch="workbench.loadFromUrl"
        />
        <ResultEditor
          v-model="workbench.result.value"
          class="bg-cp-bg-container"
          :phase="workbench.phase.value"
          :running="workbench.running.value"
          :loading="initializing"
          :can-generate="workbench.canGenerate.value && !initializing"
          :saving="workbench.historySaving.value"
          @generate="workbench.generate"
          @retry="workbench.retry"
          @stop="workbench.stop"
          @save="workbench.saveCurrent"
          @continue="workbench.continueWith"
        />
      </div>
    </BaseCard>

    <ExecutionFacts :facts="workbench.facts.value" />
  </div>
</template>
