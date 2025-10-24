/**
 * Base Recommendations Component
 * Renders a recommendation widget with specified layout
 */

import React, { useEffect, useRef } from 'react';
import type { RecommendationProps } from '../types';

export const Recommendations: React.FC<RecommendationProps> = ({
  apiKey,
  tenantId,
  apiUrl,
  productId,
  count = 5,
  type = 'similar',
  layout = 'grid',
  theme = 'light',
  realTime = false,
  userId,
  testId,
  autoTrack = true,
  debug = false,
  onProductClick,
  onProductImpression,
  className = '',
  style = {},
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const isInitialized = useRef(false);

  useEffect(() => {
    if (!containerRef.current || isInitialized.current) return;

    // Set global config if not already set
    if (!(window as any).GrooveShopConfig) {
      (window as any).GrooveShopConfig = {
        apiKey,
        tenantId,
        apiUrl,
        autoTrack,
        debug,
      };
    }

    // Load widget script if not loaded
    if (!(window as any).GrooveShopRecommendations) {
      const script = document.createElement('script');
      script.src = apiUrl
        ? `${apiUrl}/widget.js`
        : '/widget.js'; // Adjust path as needed
      script.async = true;
      document.head.appendChild(script);
    }

    // Set up event listeners
    if (onProductClick) {
      const widget = (window as any).GrooveShopRecommendations;
      if (widget?.on) {
        const unsubscribe = widget.on('click', (event: any) => {
          const product = {
            entity_id: event.productId,
            attributes: {
              name: event.metadata?.name || '',
              price: event.metadata?.price || 0,
              ...event.metadata,
            },
          };
          onProductClick(product);
        });

        return () => unsubscribe();
      }
    }

    if (onProductImpression) {
      const widget = (window as any).GrooveShopRecommendations;
      if (widget?.on) {
        const unsubscribe = widget.on('impression', (event: any) => {
          // Get products from event metadata
          onProductImpression(event.metadata?.products || []);
        });

        return () => unsubscribe();
      }
    }

    isInitialized.current = true;
  }, [apiKey, tenantId, apiUrl, autoTrack, debug, onProductClick, onProductImpression]);

  return (
    <div
      ref={containerRef}
      data-grooveshop-recommendations
      data-product-id={productId}
      data-count={count}
      data-type={type}
      data-layout={layout}
      data-theme={theme}
      data-real-time={realTime}
      data-user-id={userId}
      data-test-id={testId}
      className={className}
      style={style}
    />
  );
};
