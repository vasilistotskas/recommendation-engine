import { defineNuxtModule, createResolver, addComponent, addImports, addPlugin } from '@nuxt/kit'
import { defu } from 'defu'

export interface ModuleOptions {
  /**
   * Your GrooveShop API key (starts with pk_)
   */
  apiKey: string

  /**
   * Your GrooveShop tenant ID
   */
  tenantId: string

  /**
   * API URL
   * @default 'https://api.grooveshop.com'
   */
  apiUrl?: string

  /**
   * Enable automatic tracking
   * @default true
   */
  autoTrack?: boolean

  /**
   * Enable debug mode
   * @default false
   */
  debug?: boolean

  /**
   * Enable lazy loading
   * @default true
   */
  lazyLoad?: boolean

  /**
   * Enable prefetching on hover
   * @default true
   */
  prefetch?: boolean

  /**
   * Cache timeout in milliseconds
   * @default 300000 (5 minutes)
   */
  cacheTimeout?: number

  /**
   * Enable real-time updates via WebSocket
   * @default false
   */
  realtime?: boolean

  /**
   * WebSocket URL
   * @default 'wss://api.grooveshop.com/ws'
   */
  wsUrl?: string
}

export interface ModuleHooks {
  'grooveshop:ready': () => void
}

export default defineNuxtModule<ModuleOptions>({
  meta: {
    name: '@grooveshop/nuxt-recommendations',
    configKey: 'grooveshop',
    compatibility: {
      nuxt: '>=3.0.0'
    }
  },

  defaults: {
    apiUrl: 'https://api.grooveshop.com',
    autoTrack: true,
    debug: false,
    lazyLoad: true,
    prefetch: true,
    cacheTimeout: 300000,
    realtime: false,
    wsUrl: 'wss://api.grooveshop.com/ws'
  },

  setup(options, nuxt) {
    const resolver = createResolver(import.meta.url)

    // Validate required options (skip in prepare/stub mode)
    const isPrepare = process.argv.includes('prepare') || process.argv.includes('--stub')

    if (!isPrepare) {
      if (!options.apiKey) {
        throw new Error('[GrooveShop] apiKey is required. Get yours at https://dashboard.grooveshop.com')
      }

      if (!options.tenantId) {
        throw new Error('[GrooveShop] tenantId is required. Get yours at https://dashboard.grooveshop.com')
      }
    }

    // Expose module options to runtime
    nuxt.options.runtimeConfig.public.grooveshop = defu(
      nuxt.options.runtimeConfig.public.grooveshop as any,
      {
        apiKey: options.apiKey,
        tenantId: options.tenantId,
        apiUrl: options.apiUrl,
        autoTrack: options.autoTrack,
        debug: options.debug,
        lazyLoad: options.lazyLoad,
        prefetch: options.prefetch,
        cacheTimeout: options.cacheTimeout,
        realtime: options.realtime,
        wsUrl: options.wsUrl
      }
    )

    // Add GrooveShop widget script to head
    nuxt.options.app.head.script = nuxt.options.app.head.script || []
    nuxt.options.app.head.script.push({
      src: 'https://cdn.grooveshop.com/recommendations/v1/widget.js',
      defer: true
    })

    // Preconnect to API and CDN for better performance
    nuxt.options.app.head.link = nuxt.options.app.head.link || []
    nuxt.options.app.head.link.push(
      {
        rel: 'preconnect',
        href: 'https://cdn.grooveshop.com'
      },
      {
        rel: 'preconnect',
        href: 'https://api.grooveshop.com'
      },
      {
        rel: 'dns-prefetch',
        href: 'https://cdn.grooveshop.com'
      },
      {
        rel: 'dns-prefetch',
        href: 'https://api.grooveshop.com'
      }
    )

    // Register plugin to initialize GrooveShop
    addPlugin(resolver.resolve('./runtime/plugin'))

    // Register components for auto-import
    addComponent({
      name: 'GrooveshopRecommendations',
      filePath: resolver.resolve('./runtime/components/GrooveshopRecommendations.vue')
    })

    addComponent({
      name: 'GrooveshopCarousel',
      filePath: resolver.resolve('./runtime/components/GrooveshopCarousel.vue')
    })

    addComponent({
      name: 'GrooveshopGrid',
      filePath: resolver.resolve('./runtime/components/GrooveshopGrid.vue')
    })

    addComponent({
      name: 'GrooveshopList',
      filePath: resolver.resolve('./runtime/components/GrooveshopList.vue')
    })

    // Register composables for auto-import
    addImports({
      name: 'useGrooveshop',
      from: resolver.resolve('./runtime/composables/useGrooveshop')
    })

    // Log module initialization
    if (options.debug) {
      nuxt.hook('ready', () => {
        console.log('[GrooveShop] Module initialized')
        console.log('[GrooveShop] API Key:', options.apiKey?.substring(0, 10) + '...')
        console.log('[GrooveShop] Tenant ID:', options.tenantId)
      })
    }

    // Call custom hook when module is ready
    nuxt.hook('modules:done', async () => {
      await nuxt.callHook('grooveshop:ready' as any)
    })
  }
})
