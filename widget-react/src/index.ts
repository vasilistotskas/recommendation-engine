/**
 * GrooveShop Recommendations React Components
 * @module @grooveshop/recommendations-react
 */

// Components
export { Recommendations } from './components/Recommendations';
export { RecommendationCarousel } from './components/RecommendationCarousel';
export { RecommendationGrid } from './components/RecommendationGrid';
export { RecommendationList } from './components/RecommendationList';

// Context
export { RecommendationProvider, useRecommendations } from './context/RecommendationContext';

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

export type { RecommendationCarouselProps } from './components/RecommendationCarousel';
export type { RecommendationGridProps } from './components/RecommendationGrid';
export type { RecommendationListProps } from './components/RecommendationList';
export type { RecommendationProviderProps } from './context/RecommendationContext';
