import { defineNuxtPlugin, useRuntimeConfig } from '#app'

export default defineNuxtPlugin({
  name: 'grooveshop-recommendations',
  enforce: 'pre',
  setup() {
    const config = useRuntimeConfig()
    const grooveshopConfig = config.public.grooveshop

    // Initialize GrooveShop on client side
    if (import.meta.client) {
      // Wait for GrooveShop widget to load
      const initGrooveShop = () => {
        if (typeof window !== 'undefined' && (window as any).GrooveShopRecommendations) {
          (window as any).GrooveShopRecommendations.init({
            apiKey: grooveshopConfig.apiKey,
            tenantId: grooveshopConfig.tenantId,
            apiUrl: grooveshopConfig.apiUrl,
            autoTrack: grooveshopConfig.autoTrack,
            debug: grooveshopConfig.debug,
            lazyLoad: grooveshopConfig.lazyLoad,
            prefetch: grooveshopConfig.prefetch,
            cacheTimeout: grooveshopConfig.cacheTimeout,
            realtime: grooveshopConfig.realtime,
            wsUrl: grooveshopConfig.wsUrl
          })

          if (grooveshopConfig.debug) {
            console.log('[GrooveShop] Widget initialized')
          }
        } else {
          // Retry if widget not loaded yet
          setTimeout(initGrooveShop, 100)
        }
      }

      // Initialize when DOM is ready
      if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initGrooveShop)
      } else {
        initGrooveShop()
      }
    }
  }
})
