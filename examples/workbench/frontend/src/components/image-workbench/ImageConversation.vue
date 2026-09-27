<script setup lang="ts">
import type { ImageTurn } from '../../types'
import { BaseButton, BaseMarkdown } from '@codex-proxy/ui'
import { CircleAlert, LoaderCircle, RotateCcw } from '@lucide/vue'
import { useTemplateRef, watch } from 'vue'
import ImageComposer from './ImageComposer.vue'

const props = defineProps<{
  turns: ImageTurn[]
  attachment: string
  attachmentName: string
  disabled: boolean
  running: boolean
  canSend: boolean
  error: string
}>()
defineEmits<{ send: [], stop: [], retry: [], upload: [file?: File], remove: [], select: [id: number] }>()
const prompt = defineModel<string>({ required: true })
const timeline = useTemplateRef('timeline')
let followTail = true
function scrolled() {
  const element = timeline.value
  if (element)
    followTail = element.scrollHeight - element.scrollTop - element.clientHeight < 80
}
watch(() => [props.turns.length, props.turns.at(-1)?.text, props.turns.at(-1)?.status], (next, previous) => {
  const element = timeline.value
  if (element && (followTail || next[0] !== previous?.[0]))
    element.scrollTop = element.scrollHeight
}, { flush: 'post' })
</script>

<template>
  <section class="flex min-h-96 min-w-0 flex-col gap-4 p-4 sm:p-5 md:min-h-0" aria-labelledby="image-conversation-title">
    <header class="flex shrink-0 items-center justify-between gap-2">
      <h2 id="image-conversation-title" class="m-0 text-cp-sm font-emphasis">
        对话
      </h2>
      <span class="text-cp-xs text-cp-text-tertiary">{{ turns.length ? `${turns.length} 轮` : '当前页面会话' }}</span>
    </header>
    <div ref="timeline" class="cp-scrollbar min-h-0 flex-1 overflow-y-auto p-1 -m-1" aria-label="图片对话记录" @scroll="scrolled">
      <div v-if="!turns.length" class="flex flex-col gap-3 py-3">
        <p class="m-0 text-cp-sm leading-relaxed text-cp-text-secondary">
          先说说你的想法，生成后接着描述要修改的地方
        </p>
        <button v-for="idea in ['一只坐在窗边的橘猫，水彩风格', '为一家咖啡店设计极简标志']" :key="idea" type="button" class="rounded-cp bg-cp-fill-quaternary p-3 text-left text-cp-xs text-cp-text-secondary hover:bg-cp-fill-secondary focus-visible:outline-2 focus-visible:outline-cp-primary" :disabled="disabled" @click="prompt = idea">
          {{ idea }}
        </button>
      </div>
      <ol v-else class="m-0 grid list-none gap-6 p-0">
        <li v-for="(turn, index) in turns" :key="turn.id" class="grid gap-3">
          <div class="ml-6 grid gap-2 rounded-cp-lg bg-cp-fill-tertiary px-3 py-2.5">
            <img v-if="turn.attachment" :src="turn.attachment" alt="发送的参考图" class="size-16 rounded-cp object-contain">
            <p class="m-0 text-cp-sm leading-relaxed break-words whitespace-pre-wrap">
              {{ turn.prompt }}
            </p>
          </div>
          <div class="grid gap-2 pr-2 text-cp-sm leading-relaxed">
            <BaseMarkdown v-if="turn.text" :source="turn.text" />
            <button v-if="turn.imageUrl" type="button" class="flex w-fit max-w-full items-center gap-2 rounded-cp bg-cp-fill-quaternary p-2 text-left hover:bg-cp-fill-secondary focus-visible:outline-2 focus-visible:outline-cp-primary" @click="$emit('select', turn.id)">
              <img :src="turn.imageUrl" alt="" class="size-10 rounded-cp object-contain">
              <span class="text-cp-xs text-cp-text-secondary">查看本轮图片</span>
            </button>
            <span v-if="turn.status === 'running'" class="inline-flex items-center gap-2 text-cp-xs text-cp-text-tertiary" role="status"><LoaderCircle class="size-3.5 animate-spin motion-reduce:animate-none" />正在处理</span>
            <p v-else-if="turn.status === 'error'" class="m-0 text-cp-xs break-words text-cp-error-text" role="alert">
              {{ turn.error }}
            </p>
            <span v-else-if="turn.status === 'cancelled'" class="text-cp-xs text-cp-text-tertiary">已停止，本轮未加入上下文</span>
            <BaseButton v-if="index === turns.length - 1 && ['error', 'cancelled'].includes(turn.status)" size="sm" class="w-fit" :disabled="disabled" @click="$emit('retry')">
              <template #icon>
                <RotateCcw class="size-3.5" />
              </template>重试本轮
            </BaseButton>
          </div>
        </li>
      </ol>
    </div>
    <p v-if="error" class="m-0 flex shrink-0 items-start gap-2 text-cp-xs text-cp-error-text" role="alert">
      <CircleAlert class="mt-0.5 size-3.5 shrink-0" />{{ error }}
    </p>
    <ImageComposer v-model="prompt" :attachment="attachment" :attachment-name="attachmentName" :disabled="disabled" :running="running" :can-send="canSend" :continuing="Boolean(turns.length)" @send="$emit('send')" @stop="$emit('stop')" @upload="$emit('upload', $event)" @remove="$emit('remove')" />
  </section>
</template>
