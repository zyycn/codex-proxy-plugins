<script setup lang="ts">
import { BaseButton, BaseIconButton, BaseTextarea } from '@codex-proxy/ui'
import { ArrowUp, Paperclip, Square, X } from '@lucide/vue'

const props = defineProps<{
  attachment: string
  attachmentName: string
  disabled: boolean
  running: boolean
  canSend: boolean
  continuing: boolean
}>()
const emit = defineEmits<{ send: [], stop: [], upload: [file?: File], remove: [] }>()
const prompt = defineModel<string>({ required: true })
function upload(event: Event) {
  const input = event.target as HTMLInputElement
  emit('upload', input.files?.[0])
  input.value = ''
}
function keydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
    event.preventDefault()
    if (props.canSend && !props.disabled)
      emit('send')
  }
}
</script>

<template>
  <div class="grid shrink-0 gap-3 rounded-cp-lg bg-cp-fill-quaternary p-3">
    <div v-if="attachment" class="flex min-w-0 items-center gap-2">
      <img :src="attachment" alt="本轮参考图" class="size-10 shrink-0 rounded-cp object-contain">
      <span class="min-w-0 flex-1 truncate text-cp-xs text-cp-text-secondary">{{ attachmentName }}</span>
      <BaseIconButton label="移除参考图" :disabled="disabled" @click="emit('remove')">
        <X class="size-4" />
      </BaseIconButton>
    </div>
    <BaseTextarea v-model="prompt" :rows="3" resize="none" maxlength="4000" :disabled="disabled" aria-label="图片对话消息" :placeholder="continuing ? '继续描述修改，例如：给它加一条红围巾' : '描述想要的图片，也可以附上参考图'" @keydown="keydown" />
    <div class="flex items-center justify-between gap-2">
      <label for="image-reference" class="relative inline-flex cursor-pointer items-center gap-1.5 rounded-cp px-2 py-1.5 text-cp-xs text-cp-text-secondary hover:bg-cp-fill-secondary focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-cp-primary" :class="{ 'pointer-events-none opacity-50': disabled }" title="PNG / JPEG / WebP，最大 4 MiB">
        <Paperclip class="size-4" aria-hidden="true" />参考图
        <input id="image-reference" type="file" accept="image/png,image/jpeg,image/webp" class="absolute inset-0 w-full cursor-pointer opacity-0" :disabled="disabled" aria-label="添加参考图" @change="upload">
      </label>
      <BaseButton v-if="running" size="sm" @click="emit('stop')">
        <template #icon>
          <Square class="size-3.5" />
        </template>停止
      </BaseButton>
      <BaseButton v-else size="sm" variant="primary" :disabled="disabled || !canSend" @click="emit('send')">
        <template #icon>
          <ArrowUp class="size-4" />
        </template>发送
      </BaseButton>
    </div>
  </div>
</template>
