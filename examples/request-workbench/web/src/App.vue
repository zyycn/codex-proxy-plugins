<script setup lang="ts">
import EchoForm from './components/EchoForm.vue'
import MiddlewareStatus from './components/MiddlewareStatus.vue'
</script>

<template>
  <main class="grid items-start gap-4 font-sans text-cp-text lg:grid-cols-2">
    <MiddlewareStatus />
    <EchoForm />
  </main>
</template>
