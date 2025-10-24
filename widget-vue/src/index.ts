/**
 * GrooveShop Recommendations Vue Components
 * @module @grooveshop/recommendations-vue
 */

// Components
export { default as Recommendations } from './components/Recommendations.vue';
export { default as RecommendationCarousel } from './components/RecommendationCarousel.vue';
export { default as RecommendationGrid } from './components/RecommendationGrid.vue';
export { default as RecommendationList } from './components/RecommendationList.vue';

// Composables
export { useRecommendations } from './composables/useRecommendations';

// Types
export type {
  Product,
  RecommendationType,
  LayoutType,
  ThemeType,
  RecommendationConfig,
  RecommendationProps,
  AnalyticsEvent,
  EventCallback,
} from './types';
