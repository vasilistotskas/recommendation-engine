/**
 * React Context for GrooveShop Recommendations
 */

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import type { RecommendationConfig, EventCallback } from '../types';

interface RecommendationContextValue {
  config: RecommendationConfig;
  on: (eventType: string, callback: EventCallback) => () => void;
  isInitialized: boolean;
}

const RecommendationContext = createContext<RecommendationContextValue | null>(null);

export interface RecommendationProviderProps {
  config: RecommendationConfig;
  children: React.ReactNode;
}

export const RecommendationProvider: React.FC<RecommendationProviderProps> = ({
  config,
  children,
}) => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [widget, setWidget] = useState<any>(null);

  useEffect(() => {
    // Set global config
    (window as any).GrooveShopConfig = {
      apiKey: config.apiKey,
      tenantId: config.tenantId,
      apiUrl: config.apiUrl,
      autoTrack: config.autoTrack !== false,
      debug: config.debug || false,
    };

    // Widget auto-initializes on script load
    // Check if already initialized
    if ((window as any).GrooveShopRecommendations) {
      setWidget((window as any).GrooveShopRecommendations);
      setIsInitialized(true);
    }
  }, [config]);

  const on = useCallback(
    (eventType: string, callback: EventCallback): (() => void) => {
      if (!widget) {
        console.warn('Widget not initialized yet');
        return () => {};
      }

      return widget.on(eventType, callback);
    },
    [widget]
  );

  const value: RecommendationContextValue = {
    config,
    on,
    isInitialized,
  };

  return (
    <RecommendationContext.Provider value={value}>
      {children}
    </RecommendationContext.Provider>
  );
};

export const useRecommendations = (): RecommendationContextValue => {
  const context = useContext(RecommendationContext);
  if (!context) {
    throw new Error('useRecommendations must be used within a RecommendationProvider');
  }
  return context;
};
