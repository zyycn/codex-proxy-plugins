<script setup lang="ts">
import { BaseButton, BaseCard, BaseForm, BaseFormItem, BaseInput } from '@codex-proxy/ui'
import { shallowRef } from 'vue'
import { isRecord, request } from '../host'

const message = shallowRef('你好，插件')
const reply = shallowRef<string>()
const pending = shallowRef(false)
const error = shallowRef('')

async function submit() {
  if (pending.value)
    return
  error.value = ''
  reply.value = undefined
  if (new TextEncoder().encode(message.value).byteLength > 256) {
    error.value = '消息不能超过 256 字节'
    return
  }
  pending.value = true
  try {
    const value = await request('POST', 'echo', { message: message.value })
    if (!isRecord(value) || typeof value.message !== 'string')
      throw new Error('插件返回的消息格式无效')
    reply.value = value.message
  }
  catch (cause) {
    error.value = cause instanceof Error ? cause.message : '调用失败'
  }
  finally {
    pending.value = false
  }
}
</script>

<template>
  <BaseCard title="页面调用" padding="compact">
    <BaseForm @submit="submit">
      <BaseFormItem label="消息" :error="error" required>
        <BaseInput v-model="message" :disabled="pending" required maxlength="256" autocomplete="off" />
      </BaseFormItem>
      <div class="flex justify-start">
        <BaseButton type="submit" variant="primary" :loading="pending">
          发送到插件
        </BaseButton>
      </div>
    </BaseForm>
    <p v-if="reply !== undefined" class="mt-4 mb-0 text-cp-text wrap-anywhere" role="status">
      {{ reply }}
    </p>
  </BaseCard>
</template>
