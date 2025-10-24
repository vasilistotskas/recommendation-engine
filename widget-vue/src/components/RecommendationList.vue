<template>
  <Recommendations v-bind="$props" layout="list" @product-click="emit('product-click', $event)" @product-impression="emit('product-impression', $event)" />
</template>

<script setup lang="ts">
import Recommendations from './Recommendations.vue';
import type { RecommendationProps, Product } from '../types';

interface Props extends Omit<RecommendationProps, 'layout'> {
  className?: string;
  style?: Record<string, any>;
}

defineProps<Props>();

const emit = defineEmits<{
  'product-click': [product: Product];
  'product-impression': [products: Product[]];
}>();
</script>
