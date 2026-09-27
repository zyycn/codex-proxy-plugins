<script setup lang="ts">
import type { ImageTurn } from '../../types'
import { Image, LoaderCircle } from '@lucide/vue'

defineProps<{ images: ImageTurn[], selected?: ImageTurn, running: boolean }>()
defineEmits<{ select: [id: number] }>()
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-col gap-4 bg-cp-fill-quaternary p-4 sm:p-5" aria-labelledby="image-canvas-title">
    <header class="flex shrink-0 items-center justify-between gap-3">
      <h2 id="image-canvas-title" class="m-0 inline-flex items-center gap-2 text-cp-sm font-emphasis">
        画布
        <span v-if="selected" class="rounded-cp bg-cp-bg-container px-2 py-0.5 font-mono text-cp-xs text-cp-text-secondary">{{ images.findIndex(image => image.id === selected?.id) + 1 }} / {{ images.length }}</span>
      </h2>
      <span class="inline-flex items-center gap-1.5 text-cp-xs text-cp-text-tertiary" role="status">
        <template v-if="running"><LoaderCircle class="size-3.5 animate-spin motion-reduce:animate-none" />正在创作</template>
        <template v-else-if="selected">右键或长按保存</template>
      </span>
    </header>
    <div class="relative flex min-h-72 flex-1 items-center justify-center overflow-hidden rounded-cp-lg bg-cp-bg-container md:min-h-0">
      <img v-if="selected" :src="selected.imageUrl" :alt="`第 ${images.findIndex(image => image.id === selected?.id) + 1} 版图片`" class="absolute inset-0 size-full object-contain">
      <div v-else class="flex flex-col items-center gap-3 px-5 py-10 text-center">
        <Image class="mb-2 size-10 text-cp-text-quaternary" :stroke-width="1" aria-hidden="true" />
        <p class="m-0 text-cp-lg font-emphasis">
          从一句描述开始
        </p>
        <p class="m-0 max-w-64 text-cp-sm leading-relaxed text-cp-text-tertiary">
          图片会出现在这里，继续对话就能修改细节
        </p>
      </div>
    </div>
    <div v-if="images.length" class="flex shrink-0 gap-2 overflow-x-auto p-1 -m-1" aria-label="图片版本">
      <button
        v-for="(image, index) in images" :key="image.id" type="button"
        class="relative size-14 shrink-0 overflow-hidden rounded-cp bg-cp-bg-container outline-none transition-shadow focus-visible:ring-2 focus-visible:ring-cp-primary motion-reduce:transition-none"
        :class="selected?.id === image.id ? 'ring-2 ring-cp-primary' : 'opacity-60 hover:opacity-100'"
        :aria-label="`查看第 ${index + 1} 版图片`" :aria-pressed="selected?.id === image.id" :title="image.prompt"
        @click="$emit('select', image.id)"
      >
        <img :src="image.imageUrl" alt="" class="size-full object-contain">
        <span class="absolute right-0 bottom-0 rounded-tl-cp bg-cp-bg-container px-1.5 font-mono text-cp-xs text-cp-text-secondary">{{ index + 1 }}</span>
      </button>
    </div>
  </section>
</template>
