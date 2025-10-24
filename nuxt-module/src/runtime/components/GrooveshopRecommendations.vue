<template>
  <div
    ref="containerRef"
    data-grooveshop-recommendations
    :data-type="type"
    :data-product-id="productId"
    :data-user-id="userId"
    :data-count="count"
    :data-layout="layout"
    :data-theme="theme"
    :data-real-time="realTime ? 'true' : 'false'"
  />
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

export interface GrooveshopRecommendationsProps {
  type?: 'similar' | 'trending' | 'bundle' | 'personalized' | 'complement' | 'recently-viewed' | 'auto'
  productId?: string | number
  userId?: string | number
  count?: number
  layout?: 'carousel' | 'grid' | 'list'
  theme?: 'light' | 'dark' | 'minimal'
  realTime?: boolean
}

const props = withDefaults(defineProps<GrooveshopRecommendationsProps>(), {
  type: 'similar',
  count: 5,
  layout: 'carousel',
  theme: 'light',
  realTime: false
})

const containerRef = ref<HTMLElement>()

onMounted(() => {
  // Widget will auto-initialize via plugin
  // This component just provides the Vue-friendly wrapper
})
</script>
