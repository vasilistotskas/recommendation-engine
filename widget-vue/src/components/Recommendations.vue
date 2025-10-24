<template>
  <div
    ref="containerRef"
    data-grooveshop-recommendations
    :data-product-id="productId"
    :data-count="count"
    :data-type="type"
    :data-layout="layout"
    :data-theme="theme"
    :data-real-time="realTime"
    :data-user-id="userId"
    :data-test-id="testId"
    :class="className"
    :style="style"
  />
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { RecommendationProps, Product } from '../types';

interface Props extends RecommendationProps {
  className?: string;
  style?: Record<string, any>;
}

const props = withDefaults(defineProps<Props>(), {
  count: 5,
  type: 'similar',
  layout: 'grid',
  theme: 'light',
  realTime: false,
  autoTrack: true,
  debug: false,
  className: '',
  style: () => ({}),
});

const emit = defineEmits<{
  'product-click': [product: Product];
  'product-impression': [products: Product[]];
}>();

const containerRef = ref<HTMLElement | null>(null);

onMounted(() => {
  // Set global config if not already set
  if (!(window as any).GrooveShopConfig) {
    (window as any).GrooveShopConfig = {
      apiKey: props.apiKey,
      tenantId: props.tenantId,
      apiUrl: props.apiUrl,
      autoTrack: props.autoTrack,
      debug: props.debug,
    };
  }

  // Load widget script if not loaded
  if (!(window as any).GrooveShopRecommendations) {
    const script = document.createElement('script');
    script.src = props.apiUrl
      ? `${props.apiUrl}/widget.js`
      : '/widget.js';
    script.async = true;
    document.head.appendChild(script);
  }

  // Set up event listeners
  const widget = (window as any).GrooveShopRecommendations;
  if (widget?.on) {
    const unsubscribeClick = widget.on('click', (event: any) => {
      const product: Product = {
        entity_id: event.productId,
        attributes: {
          name: event.metadata?.name || '',
          price: event.metadata?.price || 0,
          ...event.metadata,
        },
      };
      emit('product-click', product);
    });

    const unsubscribeImpression = widget.on('impression', (event: any) => {
      emit('product-impression', event.metadata?.products || []);
    });

    // Cleanup
    return () => {
      unsubscribeClick();
      unsubscribeImpression();
    };
  }
});
</script>
