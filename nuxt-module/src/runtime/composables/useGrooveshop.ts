import { useRuntimeConfig } from '#app'
import { ref, computed } from 'vue'

export interface Product {
  entity_id: string
  attributes: {
    name: string
    price: number
    image_url: string
    url: string
    discount_price?: number
    currency?: string
    rating?: number
    review_count?: number
    in_stock?: boolean
    category?: string
    tags?: string[]
  }
}

export interface ClickEvent {
  type: 'click'
  productId: string
  position: number
  sourceType: string
  sourceProductId?: string
  timestamp: number
  metadata?: Record<string, any>
}

export interface ImpressionEvent {
  type: 'impression'
  productIds: string[]
  sourceType: string
  timestamp: number
}

/**
 * Composable for interacting with GrooveShop Recommendations
 */
export function useGrooveshop() {
  const config = useRuntimeConfig()
  const grooveshopConfig = config.public.grooveshop

  const isReady = ref(false)
  const isClient = computed(() => import.meta.client)

  /**
   * Check if GrooveShop widget is ready
   */
  const checkReady = () => {
    if (isClient.value && typeof window !== 'undefined') {
      isReady.value = !!(window as any).GrooveShopRecommendations
    }
    return isReady.value
  }

  /**
   * Get the GrooveShop widget instance
   */
  const getWidget = () => {
    if (!checkReady()) {
      console.warn('[GrooveShop] Widget not ready yet')
      return null
    }
    return (window as any).GrooveShopRecommendations
  }

  /**
   * Track a custom event
   */
  const trackEvent = (type: string, data: Record<string, any>) => {
    const widget = getWidget()
    if (widget && widget.trackEvent) {
      return widget.trackEvent(type, data)
    }
  }

  /**
   * Set user context
   */
  const setUser = (userData: { id: string; email?: string; name?: string }) => {
    const widget = getWidget()
    if (widget && widget.setUser) {
      widget.setUser(userData)
    }
  }

  /**
   * Register event listener
   */
  const on = (event: string, callback: (data: any) => void) => {
    const widget = getWidget()
    if (widget && widget.on) {
      return widget.on(event, callback)
    }
  }

  /**
   * Remove event listener
   */
  const off = (event: string, callback: (data: any) => void) => {
    const widget = getWidget()
    if (widget && widget.off) {
      widget.off(event, callback)
    }
  }

  /**
   * Refresh all widgets
   */
  const refresh = () => {
    const widget = getWidget()
    if (widget && widget.refresh) {
      return widget.refresh()
    }
  }

  /**
   * Clear cache
   */
  const clearCache = () => {
    const widget = getWidget()
    if (widget && widget.clearCache) {
      widget.clearCache()
    }
  }

  // Auto-check ready status on mount
  if (isClient.value) {
    setTimeout(checkReady, 100)
  }

  return {
    // State
    isReady,
    isClient,
    config: grooveshopConfig,

    // Methods
    checkReady,
    getWidget,
    trackEvent,
    setUser,
    on,
    off,
    refresh,
    clearCache
  }
}
