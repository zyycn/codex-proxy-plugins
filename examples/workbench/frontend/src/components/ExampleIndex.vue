<script setup lang="ts">
import type { ExampleGuide } from '../types'
import { BaseSelect } from '@codex-proxy/ui'
import { ChevronRight } from '@lucide/vue'
import { computed } from 'vue'

const props = defineProps<{ examples: ExampleGuide[] }>()
const selectedId = defineModel<string>({ required: true })
const groups = computed(() => [
  { label: '动手体验', items: props.examples.filter(item => item.group === 'interactive') },
  { label: '接入指南', items: props.examples.filter(item => item.group === 'integration') },
])
const options = computed(() => props.examples.map(item => ({
  value: item.id,
  label: item.title,
  description: item.group === 'interactive' ? '动手体验' : '接入指南',
})))
</script>

<template>
  <nav class="min-w-0 bg-cp-bg-container p-4 lg:px-3 lg:py-5" aria-label="基础示例">
    <BaseSelect v-model="selectedId" :options="options" class="lg:hidden" aria-label="选择基础示例" />
    <div class="hidden gap-6 lg:grid">
      <section v-for="group in groups" :key="group.label" class="grid gap-2" :aria-label="group.label">
        <h2 class="m-0 px-3 text-cp-xs font-normal text-cp-text-tertiary">
          {{ group.label }}
        </h2>
        <div class="grid gap-1">
          <button
            v-for="item in group.items"
            :key="item.id"
            type="button"
            class="flex min-h-10 w-full cursor-pointer items-center gap-2.5 rounded-cp border-0 px-3 py-2 text-left text-cp-sm outline-none transition-colors focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-cp-control-outline motion-reduce:transition-none"
            :class="selectedId === item.id ? 'bg-cp-primary-container font-emphasis text-cp-primary-on-container' : 'bg-transparent text-cp-text-secondary hover:bg-cp-fill-tertiary hover:text-cp-text'"
            :aria-current="selectedId === item.id ? 'true' : undefined"
            @click="selectedId = item.id"
          >
            <component :is="item.icon" class="size-4 shrink-0" aria-hidden="true" />
            <span class="min-w-0 flex-1">{{ item.title }}</span>
            <ChevronRight class="size-3.5 shrink-0" :class="selectedId === item.id ? 'opacity-70' : 'opacity-0'" aria-hidden="true" />
          </button>
        </div>
      </section>
    </div>
  </nav>
</template>
