<script setup lang="ts">
import type { WorkbenchSnapshot } from '../../api'
import { BaseEmpty, BaseIconButton } from '@codex-proxy/ui'
import { RefreshCw } from '@lucide/vue'
import { toRef } from 'vue'
import ExampleDetail from '../../components/ExampleDetail.vue'
import ExampleIndex from '../../components/ExampleIndex.vue'
import { useExampleRunner } from '../../composables/useExampleRunner'

const props = defineProps<{
  snapshot?: WorkbenchSnapshot
  loading: boolean
  error: string
  refresh: () => Promise<void>
}>()
const runner = useExampleRunner(toRef(props, 'snapshot'), props.refresh)
</script>

<template>
  <div class="flex min-w-0 flex-1 flex-col gap-3">
    <p v-if="error" class="m-0 text-cp-sm text-cp-error-text" role="alert">
      {{ error }}
    </p>
    <div class="grid min-w-0 flex-1 grid-rows-[auto_1fr] overflow-hidden rounded-cp-card bg-cp-bg-container lg:grid-cols-[12rem_minmax(0,1fr)] lg:grid-rows-1">
      <ExampleIndex
        v-model="runner.selectedId.value"
        :examples="runner.examples.value"
      />
      <ExampleDetail
        v-if="runner.selected.value"
        v-model:demo-key-id="runner.keyId.value"
        v-model:message="runner.message.value"
        :example="runner.selected.value"
        :evidence="runner.evidence.value"
        :key-options="runner.keyOptions.value"
        :run="runner.run.value"
        :busy="Boolean(runner.activeId.value)"
        @run="runner.execute"
        @stop="runner.stop"
      >
        <template #actions>
          <BaseIconButton label="刷新调用记录" :loading="loading" @click="refresh">
            <RefreshCw />
          </BaseIconButton>
        </template>
      </ExampleDetail>
      <BaseEmpty v-else :description="loading ? '正在载入示例' : '暂时无法载入示例，请刷新重试'" />
    </div>
  </div>
</template>
