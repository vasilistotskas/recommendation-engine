/**
 * Type definitions for GrooveShop Recommendations Widget
 */

export interface WidgetConfig {
  apiKey: string;
  tenantId: string;
  apiUrl?: string;
  autoTrack?: boolean;
  debug?: boolean;
  abTests?: Record<string, ABTestConfig>;
}

export interface ABTestConfig {
  variants: ABVariant[];
}

export interface ABVariant {
  name: string;
  weight: number;
  config: Partial<WidgetAttributes>;
}

export interface WidgetAttributes {
  productId?: string;
  count?: number;
  layout?: 'carousel' | 'grid' | 'list';
  type?: 'similar' | 'trending' | 'bundle' | 'personalized' | 'complement' | 'recently-viewed' | 'auto';
  theme?: 'light' | 'dark' | 'minimal';
  realTime?: boolean;
  userId?: string;
  testId?: string; // A/B test ID
}

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

export interface TrackingEvent {
  type: 'impression' | 'click' | 'view' | 'add_to_cart' | 'purchase';
  sourceProductId?: string;
  targetProductId: string;
  userId?: string;
  timestamp: number;
  metadata?: Record<string, any>;
}

export interface ApiResponse<T> {
  data?: T;
  error?: {
    code: number;
    message: string;
  };
}
