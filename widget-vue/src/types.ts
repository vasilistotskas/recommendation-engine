/**
 * Type definitions for Vue components
 */

export interface Product {
  entity_id: string;
  attributes: {
    name: string;
    price: number;
    image_url?: string;
    category?: string;
    rating?: number;
    review_count?: number;
    stock?: number;
    discount?: number;
    [key: string]: any;
  };
  score?: number;
}

export type RecommendationType =
  | 'similar'
  | 'trending'
  | 'bundle'
  | 'personalized'
  | 'complement'
  | 'recently-viewed'
  | 'auto';

export type LayoutType = 'carousel' | 'grid' | 'list';

export type ThemeType = 'light' | 'dark' | 'minimal';

export interface RecommendationConfig {
  apiKey: string;
  tenantId: string;
  apiUrl?: string;
  autoTrack?: boolean;
  debug?: boolean;
}

export interface RecommendationProps {
  apiKey: string;
  tenantId: string;
  apiUrl?: string;
  productId?: string;
  count?: number;
  type?: RecommendationType;
  layout?: LayoutType;
  theme?: ThemeType;
  realTime?: boolean;
  userId?: string;
  testId?: string;
  autoTrack?: boolean;
  debug?: boolean;
}

export interface AnalyticsEvent {
  type: 'impression' | 'click' | 'view' | 'add_to_cart' | 'purchase';
  productId: string;
  sourceProductId?: string;
  position?: number;
  score?: number;
  metadata?: Record<string, any>;
}

export type EventCallback = (event: AnalyticsEvent) => void;
