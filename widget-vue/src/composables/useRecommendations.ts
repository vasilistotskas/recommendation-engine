/**
 * Vue composable for GrooveShop Recommendations
 */

import { ref, onMounted } from 'vue';
import type { EventCallback, RecommendationConfig } from '../types';

export function useRecommendations(config?: RecommendationConfig) {
  const isInitialized = ref(false);
  const widget = ref<any>(null);

  onMounted(() => {
    // Set global config if provided
    if (config) {
      (window as any).GrooveShopConfig = {
        apiKey: config.apiKey,
        tenantId: config.tenantId,
        apiUrl: config.apiUrl,
        autoTrack: config.autoTrack !== false,
        debug: config.debug || false,
      };
    }

    // Check if widget is initialized
    if ((window as any).GrooveShopRecommendations) {
      widget.value = (window as any).GrooveShopRecommendations;
      isInitialized.value = true;
    }
  });

  const on = (eventType: string, callback: EventCallback): (() => void) => {
    if (!widget.value) {
      console.warn('Widget not initialized yet');
      return () => {};
    }

    return widget.value.on(eventType, callback);
  };

  return {
    isInitialized,
    on,
  };
}
