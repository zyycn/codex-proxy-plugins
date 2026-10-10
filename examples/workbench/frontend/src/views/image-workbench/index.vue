<script setup lang="ts">
import type { WorkbenchSnapshot } from '../../api'
import { ZCard } from '@codex-proxy/ui'
import { computed, toRef } from 'vue'
import ImageCanvas from '../../components/image-workbench/ImageCanvas.vue'
import ImageConversation from '../../components/image-workbench/ImageConversation.vue'
import ImageToolbar from '../../components/image-workbench/ImageToolbar.vue'
import { useImageWorkbench } from '../../composables/useImageWorkbench'

const props = defineProps<{ snapshot?: WorkbenchSnapshot, loading: boolean, error: string }>()
const image = useImageWorkbench(toRef(props, 'snapshot'))
const disabled = computed(() => props.loading || image.running.value || image.reading.value)
const error = computed(() => image.error.value || image.modelsError.value || props.error)
</script>

<template>
  <ZCard padding="none" class="flex min-h-0 min-w-0 flex-col overflow-hidden md:flex-1">
    <ImageToolbar
      v-model:client-key-id="image.clientKeyId.value" v-model:model-id="image.modelId.value"
      v-model:quality="image.quality.value" v-model:size="image.size.value"
      :key-options="image.keyOptions.value" :model-options="image.modelOptions.value"
      :disabled="disabled" :locked="Boolean(image.turns.value.length)" :loading="image.loading.value"
      @new="image.newConversation"
    />
    <div class="grid min-h-0 min-w-0 flex-1 md:grid-cols-[minmax(0,1fr)_22rem] xl:grid-cols-[minmax(0,1fr)_25rem]">
      <ImageCanvas :images="image.images.value" :selected="image.selectedImage.value" :running="image.running.value" @select="image.selectedId.value = $event" />
      <ImageConversation
        v-model="image.prompt.value" :turns="image.turns.value" :attachment="image.attachment.value"
        :attachment-name="image.attachmentName.value" :disabled="disabled" :running="image.running.value"
        :can-send="image.canSend.value" :error="error" :warning="image.warning.value" :phase="image.phase.value"
        @send="image.send" @stop="image.stop" @retry="image.retry"
        @upload="image.selectFile" @remove="image.clearAttachment" @select="image.selectedId.value = $event"
      />
    </div>
  </ZCard>
</template>
