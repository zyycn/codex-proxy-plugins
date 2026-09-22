<script setup lang="ts">
import { BaseButton, BaseCard } from '@codex-proxy/ui'
import { onMounted, shallowRef } from 'vue'
import { isRecord, request } from '../host'

interface Status {
  headerValue: string
  handledRequests: number
}

const status = shallowRef<Status>()
const loading = shallowRef(false)
const error = shallowRef('')

async function refresh() {
  if (loading.value)
    return
  loading.value = true
  error.value = ''
  try {
    const value = await request('GET', 'status')
    if (!isRecord(value) || typeof value.headerValue !== 'string'
      || !Number.isSafeInteger(value.handledRequests) || Number(value.handledRequests) < 0) {
      throw new Error('插件状态格式无效')
    }
    status.value = { headerValue: value.headerValue, handledRequests: Number(value.handledRequests) }
  }
  catch (cause) {
    error.value = cause instanceof Error ? cause.message : '读取状态失败'
  }
  finally {
    loading.value = false
  }
}

onMounted(refresh)
</script>

<template>
  <BaseCard title="中间件" padding="compact">
    <template #actions>
      <BaseButton size="sm" :loading="loading" @click="refresh">
        刷新
      </BaseButton>
    </template>
    <p v-if="error" class="m-0 text-cp-error-text" role="alert">
      {{ error }}
    </p>
    <dl v-else-if="status" class="m-0 grid gap-3">
      <div class="flex flex-wrap justify-between gap-x-5 gap-y-2">
        <dt class="text-cp-text-secondary">
          响应标记
        </dt>
        <dd class="m-0 wrap-anywhere">
          <code class="font-mono">x-cpr-example: {{ status.headerValue }}</code>
        </dd>
      </div>
      <div class="flex flex-wrap justify-between gap-x-5 gap-y-2">
        <dt class="text-cp-text-secondary">
          本进程处理请求
        </dt>
        <dd class="m-0">
          {{ status.handledRequests }}
        </dd>
      </div>
    </dl>
    <p v-else class="m-0 text-cp-text-secondary" role="status">
      正在读取状态
    </p>
  </BaseCard>
</template>
