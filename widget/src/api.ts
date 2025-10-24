/**
 * API client for recommendation engine
 */

import type { Product, TrackingEvent } from './types';
import type { Config } from './config';

export class ApiClient {
  private config: Config;
  private cache: Map<string, { data: Product[]; timestamp: number }>;
  private readonly CACHE_TTL = 5 * 60 * 1000; // 5 minutes

  constructor(config: Config) {
    this.config = config;
    this.cache = new Map();
  }

  /**
   * Fetch similar products
   */
  public async fetchSimilar(productId: string, count: number = 5): Promise<Product[]> {
    const cacheKey = `similar-${productId}-${count}`;

    // Check cache
    const cached = this.getFromCache(cacheKey);
    if (cached) {
      this.config.log('Cache hit:', cacheKey);
      return cached;
    }

    const widgetConfig = this.config.get();
    const url = `${widgetConfig.apiUrl}/api/v1/recommendations/${widgetConfig.tenantId}/similar/${productId}?count=${count}`;

    this.config.log('Fetching similar products:', url);

    try {
      const response = await this.fetchWithRetry(url);
      const products = await response.json() as Product[];

      this.setCache(cacheKey, products);
      return products;
    } catch (error) {
      this.config.error('Failed to fetch similar products:', error);
      return [];
    }
  }

  /**
   * Fetch trending products
   */
  public async fetchTrending(count: number = 5, category?: string): Promise<Product[]> {
    const cacheKey = `trending-${category || 'all'}-${count}`;

    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return cached;
    }

    const widgetConfig = this.config.get();
    const url = `${widgetConfig.apiUrl}/api/v1/recommendations/${widgetConfig.tenantId}/trending?count=${count}${category ? `&category=${category}` : ''}`;

    try {
      const response = await this.fetchWithRetry(url);
      const products = await response.json() as Product[];

      this.setCache(cacheKey, products);
      return products;
    } catch (error) {
      this.config.error('Failed to fetch trending products:', error);
      return [];
    }
  }

  /**
   * Fetch frequently bought together (bundle) products
   */
  public async fetchBundle(productIds: string[], count: number = 5): Promise<Product[]> {
    const cacheKey = `bundle-${productIds.join(',')}-${count}`;

    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return cached;
    }

    const widgetConfig = this.config.get();
    const url = `${widgetConfig.apiUrl}/api/v1/recommendations/${widgetConfig.tenantId}/bundle`;

    this.config.log('Fetching bundle products:', productIds);

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${widgetConfig.apiKey}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          product_ids: productIds,
          count,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      const products = await response.json() as Product[];
      this.setCache(cacheKey, products);
      return products;
    } catch (error) {
      this.config.error('Failed to fetch bundle products:', error);
      return [];
    }
  }

  /**
   * Fetch personalized recommendations for a user
   */
  public async fetchPersonalized(userId?: string, count: number = 5): Promise<Product[]> {
    const user = userId || this.getAnonymousUserId();
    const cacheKey = `personalized-${user}-${count}`;

    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return cached;
    }

    const widgetConfig = this.config.get();
    const url = `${widgetConfig.apiUrl}/api/v1/recommendations/${widgetConfig.tenantId}/personalized/${user}?count=${count}`;

    this.config.log('Fetching personalized products for user:', user);

    try {
      const response = await this.fetchWithRetry(url);
      const products = await response.json() as Product[];

      this.setCache(cacheKey, products);
      return products;
    } catch (error) {
      this.config.error('Failed to fetch personalized products:', error);
      return [];
    }
  }

  /**
   * Fetch complementary products that go well with a product
   */
  public async fetchComplement(productId: string, count: number = 5): Promise<Product[]> {
    const cacheKey = `complement-${productId}-${count}`;

    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return cached;
    }

    const widgetConfig = this.config.get();
    const url = `${widgetConfig.apiUrl}/api/v1/recommendations/${widgetConfig.tenantId}/complement/${productId}?count=${count}`;

    this.config.log('Fetching complement products:', productId);

    try {
      const response = await this.fetchWithRetry(url);
      const products = await response.json() as Product[];

      this.setCache(cacheKey, products);
      return products;
    } catch (error) {
      this.config.error('Failed to fetch complement products:', error);
      return [];
    }
  }

  /**
   * Fetch recently viewed products from localStorage
   */
  public async fetchRecentlyViewed(count: number = 5): Promise<Product[]> {
    const userId = this.getAnonymousUserId();
    const viewedIds = this.getViewHistory();

    if (viewedIds.length === 0) {
      return [];
    }

    const cacheKey = `recent-${userId}-${count}`;
    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return cached;
    }

    // Fetch product details for recently viewed IDs
    const widgetConfig = this.config.get();
    const url = `${widgetConfig.apiUrl}/api/v1/recommendations/${widgetConfig.tenantId}/batch`;

    this.config.log('Fetching recently viewed products:', viewedIds.slice(0, count));

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${widgetConfig.apiKey}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          product_ids: viewedIds.slice(0, count),
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      const products = await response.json() as Product[];
      this.setCache(cacheKey, products);
      return products;
    } catch (error) {
      this.config.error('Failed to fetch recently viewed products:', error);
      return [];
    }
  }

  /**
   * Track interaction event
   */
  public async trackInteraction(event: TrackingEvent): Promise<boolean> {
    const widgetConfig = this.config.get();
    const url = `${widgetConfig.apiUrl}/api/v1/interactions`;

    this.config.log('Tracking interaction:', event);

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${widgetConfig.apiKey}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          tenant_id: widgetConfig.tenantId,
          user_id: event.userId || this.getAnonymousUserId(),
          entity_id: event.targetProductId,
          interaction_type: event.type,
          metadata: event.metadata || {},
          timestamp: new Date().toISOString(),
        }),
      });

      return response.ok;
    } catch (error) {
      this.config.error('Failed to track interaction:', error);
      return false;
    }
  }

  /**
   * Fetch with retry logic
   */
  private async fetchWithRetry(url: string, retries: number = 3): Promise<Response> {
    const widgetConfig = this.config.get();

    for (let i = 0; i < retries; i++) {
      try {
        const response = await fetch(url, {
          method: 'GET',
          headers: {
            'Authorization': `Bearer ${widgetConfig.apiKey}`,
            'Content-Type': 'application/json',
          },
        });

        if (response.ok) {
          return response;
        }

        if (response.status === 429) {
          // Rate limited, wait and retry
          await this.sleep(1000 * Math.pow(2, i));
          continue;
        }

        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      } catch (error) {
        if (i === retries - 1) {
          throw error;
        }
        await this.sleep(1000 * Math.pow(2, i));
      }
    }

    throw new Error('Max retries exceeded');
  }

  /**
   * Cache helpers
   */
  private getFromCache(key: string): Product[] | null {
    const cached = this.cache.get(key);
    if (!cached) return null;

    const now = Date.now();
    if (now - cached.timestamp > this.CACHE_TTL) {
      this.cache.delete(key);
      return null;
    }

    return cached.data;
  }

  private setCache(key: string, data: Product[]): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
    });
  }

  /**
   * Get or create anonymous user ID
   */
  private getAnonymousUserId(): string {
    const key = 'gs_anonymous_id';
    let userId = localStorage.getItem(key);

    if (!userId) {
      userId = `anon_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      localStorage.setItem(key, userId);
    }

    return userId;
  }

  /**
   * Sleep helper
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Get view history from localStorage
   */
  private getViewHistory(): string[] {
    const key = 'gs_view_history';
    try {
      const history = localStorage.getItem(key);
      return history ? JSON.parse(history) : [];
    } catch {
      return [];
    }
  }

  /**
   * Add product to view history
   */
  public addToViewHistory(productId: string): void {
    const key = 'gs_view_history';
    const history = this.getViewHistory();

    // Remove if already exists
    const filtered = history.filter(id => id !== productId);

    // Add to beginning
    filtered.unshift(productId);

    // Keep only last 50
    const trimmed = filtered.slice(0, 50);

    try {
      localStorage.setItem(key, JSON.stringify(trimmed));
    } catch (error) {
      this.config.error('Failed to save view history:', error);
    }
  }

  /**
   * Get cart product IDs (integration point for e-commerce platforms)
   */
  public getCartProductIds(): string[] {
    // Try common e-commerce cart patterns
    const key = 'gs_cart_items';

    try {
      // Custom implementation
      const cart = localStorage.getItem(key);
      if (cart) {
        const items = JSON.parse(cart);
        return Array.isArray(items) ? items.map((item: any) => item.productId || item.id) : [];
      }

      // WooCommerce cart
      const wooCart = localStorage.getItem('wc_cart_hash');
      if (wooCart && (window as any).wc_cart_fragments) {
        // Parse WooCommerce cart fragments
        return []; // TODO: Implement WooCommerce cart parsing
      }

      // Shopify cart
      if ((window as any).Shopify?.cart) {
        const shopifyCart = (window as any).Shopify.cart.items || [];
        return shopifyCart.map((item: any) => item.product_id.toString());
      }

      return [];
    } catch (error) {
      this.config.error('Failed to get cart items:', error);
      return [];
    }
  }
}
