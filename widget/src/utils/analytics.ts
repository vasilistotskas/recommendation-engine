/**
 * Analytics Integration Module
 *
 * Supports Google Analytics 4, Segment, Facebook Pixel, and custom callbacks
 */

export type AnalyticsEvent = {
  type: 'impression' | 'click' | 'view' | 'add_to_cart' | 'purchase';
  productId: string;
  sourceProductId?: string;
  position?: number;
  score?: number;
  metadata?: Record<string, any>;
};

export type EventCallback = (event: AnalyticsEvent) => void;

export class AnalyticsIntegration {
  private callbacks: Map<string, Set<EventCallback>> = new Map();

  /**
   * Register a custom event callback
   */
  public on(eventType: string, callback: EventCallback): () => void {
    if (!this.callbacks.has(eventType)) {
      this.callbacks.set(eventType, new Set());
    }

    this.callbacks.get(eventType)!.add(callback);

    // Return unsubscribe function
    return () => {
      const callbacks = this.callbacks.get(eventType);
      if (callbacks) {
        callbacks.delete(callback);
      }
    };
  }

  /**
   * Track an event
   */
  public track(event: AnalyticsEvent): void {
    // Call custom callbacks
    this.triggerCallbacks(event.type, event);
    this.triggerCallbacks('*', event); // Wildcard listeners

    // Send to integrations
    this.sendToGoogleAnalytics(event);
    this.sendToSegment(event);
    this.sendToFacebookPixel(event);
  }

  /**
   * Trigger registered callbacks
   */
  private triggerCallbacks(eventType: string, event: AnalyticsEvent): void {
    const callbacks = this.callbacks.get(eventType);
    if (!callbacks) return;

    callbacks.forEach((callback) => {
      try {
        callback(event);
      } catch (error) {
        console.error('[Analytics] Callback error:', error);
      }
    });
  }

  /**
   * Send event to Google Analytics 4
   */
  private sendToGoogleAnalytics(event: AnalyticsEvent): void {
    if (typeof (window as any).gtag !== 'function') {
      return; // GA not installed
    }

    const eventName = this.mapEventName(event.type);

    (window as any).gtag('event', eventName, {
      product_id: event.productId,
      source_product_id: event.sourceProductId,
      position: event.position,
      score: event.score,
      event_category: 'recommendations',
      event_label: 'grooveshop_widget',
      ...event.metadata,
    });
  }

  /**
   * Send event to Segment
   */
  private sendToSegment(event: AnalyticsEvent): void {
    if (typeof (window as any).analytics?.track !== 'function') {
      return; // Segment not installed
    }

    const eventName = this.mapEventName(event.type);

    (window as any).analytics.track(eventName, {
      product_id: event.productId,
      source_product_id: event.sourceProductId,
      position: event.position,
      score: event.score,
      widget: 'grooveshop_recommendations',
      ...event.metadata,
    });
  }

  /**
   * Send event to Facebook Pixel
   */
  private sendToFacebookPixel(event: AnalyticsEvent): void {
    if (typeof (window as any).fbq !== 'function') {
      return; // Facebook Pixel not installed
    }

    // Map to Facebook standard events
    const fbEventMap: Record<string, string> = {
      view: 'ViewContent',
      click: 'ViewContent',
      add_to_cart: 'AddToCart',
      purchase: 'Purchase',
    };

    const fbEvent = fbEventMap[event.type];
    if (!fbEvent) return;

    (window as any).fbq('track', fbEvent, {
      content_ids: [event.productId],
      content_type: 'product',
      source: 'grooveshop_recommendations',
      ...event.metadata,
    });
  }

  /**
   * Map internal event types to standard analytics event names
   */
  private mapEventName(type: string): string {
    const eventMap: Record<string, string> = {
      impression: 'recommendation_impression',
      click: 'recommendation_click',
      view: 'product_view',
      add_to_cart: 'add_to_cart',
      purchase: 'purchase',
    };

    return eventMap[type] || `grooveshop_${type}`;
  }
}

/**
 * Create global analytics instance
 */
export function createAnalytics(): AnalyticsIntegration {
  return new AnalyticsIntegration();
}
