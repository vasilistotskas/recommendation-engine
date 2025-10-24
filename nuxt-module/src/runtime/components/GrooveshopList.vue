<template>
  <GrooveshopRecommendations
    v-bind="$props"
    layout="list"
  />
</template>

<script setup lang="ts">
import type { GrooveshopRecommendationsProps } from './GrooveshopRecommendations.vue'

defineProps<Omit<GrooveshopRecommendationsProps, 'layout'>>()
</script>
