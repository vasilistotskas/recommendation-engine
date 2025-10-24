# @grooveshop/nuxt-recommendations

[![npm version](https://badge.fury.io/js/@grooveshop%2Fnuxt-recommendations.svg)](https://www.npmjs.com/package/@grooveshop/nuxt-recommendations)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

AI-powered product recommendations for Nuxt 3+ applications. Seamlessly integrate personalized product recommendations into your Nuxt store.

## Features

- ✅ **Nuxt 3/4 Compatible** - Works with Nuxt 3.x and 4.x
- ✅ **Auto-Import Components** - Use components without importing
- ✅ **Auto-Import Composables** - Access `useGrooveshop()` anywhere
- ✅ **TypeScript Support** - Full type definitions included
- ✅ **SSR Ready** - Client-side initialization with server-side rendering
- ✅ **Zero Config** - Works out of the box with minimal setup
- ✅ **7 Recommendation Types** - Similar, trending, bundle, personalized, and more
- ✅ **3 Layouts** - Carousel, grid, and list views
- ✅ **Performance Optimized** - Lazy loading, prefetching, caching

## Quick Setup

### 1. Install

```bash
npm install @grooveshop/nuxt-recommendations
# or
pnpm add @grooveshop/nuxt-recommendations
# or
yarn add @grooveshop/nuxt-recommendations
```

### 2. Add to `nuxt.config.ts`

```ts
export default defineNuxtConfig({
  modules: ['@grooveshop/nuxt-recommendations'],

  grooveshop: {
    apiKey: 'pk_live_your_key_here',
    tenantId: 'your-tenant-id'
  }
})
```

### 3. Use Components

```vue
<template>
  <div>
    <!-- Similar products on product page -->
    <GrooveshopCarousel
      type="similar"
      :product-id="product.id"
      :count="5"
    />

    <!-- Trending products on homepage -->
    <GrooveshopGrid
      type="trending"
      :count="8"
    />

    <!-- Personalized recommendations -->
    <GrooveshopRecommendations
      type="personalized"
      :user-id="user.id"
      :count="6"
      layout="list"
    />
  </div>
</template>
```

That's it! 🎉

## Configuration

### Module Options

Add configuration to `nuxt.config.ts`:

```ts
export default defineNuxtConfig({
  grooveshop: {
    // Required
    apiKey: 'pk_live_your_key',       // Your public API key
    tenantId: 'your-tenant-id',       // Your store identifier

    // Optional
    apiUrl: 'https://api.grooveshop.com',  // API endpoint
    autoTrack: true,                  // Auto-track clicks/impressions
    debug: false,                     // Enable console logging
    lazyLoad: true,                   // Lazy load images
    prefetch: true,                   // Prefetch on hover
    cacheTimeout: 300000,             // Cache TTL (5 min)
    realtime: false,                  // Enable WebSocket updates
    wsUrl: 'wss://api.grooveshop.com/ws'  // WebSocket URL
  }
})
```

### Get API Credentials

1. Sign up at [dashboard.grooveshop.com](https://dashboard.grooveshop.com)
2. Create a new store
3. Copy your **API Key** (starts with `pk_`)
4. Copy your **Tenant ID**

## Components

The module auto-imports 4 components:

### `<GrooveshopRecommendations>`

Base component with all options:

```vue
<GrooveshopRecommendations
  type="similar"
  :product-id="123"
  :user-id="user.id"
  :count="5"
  layout="carousel"
  theme="light"
  :real-time="false"
/>
```

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `type` | `string` | `'similar'` | Recommendation type |
| `productId` | `string \| number` | - | Product ID for context |
| `userId` | `string \| number` | - | User ID for personalization |
| `count` | `number` | `5` | Number of products (1-20) |
| `layout` | `string` | `'carousel'` | Layout type |
| `theme` | `string` | `'light'` | Visual theme |
| `realTime` | `boolean` | `false` | Enable real-time updates |

**Recommendation Types:**
- `similar` - Similar products
- `trending` - Trending products
- `bundle` - Frequently bought together
- `personalized` - AI-powered personalized
- `complement` - Complementary products
- `recently-viewed` - Recently viewed
- `auto` - Automatically choose best type

**Layouts:**
- `carousel` - Horizontal scrolling
- `grid` - Responsive grid
- `list` - Vertical list

**Themes:**
- `light` - Light background
- `dark` - Dark background
- `minimal` - Minimal design

---

### `<GrooveshopCarousel>`

Carousel layout (horizontal scrolling):

```vue
<GrooveshopCarousel
  type="similar"
  :product-id="product.id"
  :count="5"
  theme="light"
/>
```

Same props as `<GrooveshopRecommendations>` except `layout` is fixed to `'carousel'`.

---

### `<GrooveshopGrid>`

Grid layout (responsive columns):

```vue
<GrooveshopGrid
  type="trending"
  :count="8"
  theme="dark"
/>
```

Same props as `<GrooveshopRecommendations>` except `layout` is fixed to `'grid'`.

---

### `<GrooveshopList>`

List layout (vertical stacked):

```vue
<GrooveshopList
  type="recently-viewed"
  :count="6"
/>
```

Same props as `<GrooveshopRecommendations>` except `layout` is fixed to `'list'`.

## Composables

### `useGrooveshop()`

Access the GrooveShop widget programmatically:

```vue
<script setup lang="ts">
const {
  isReady,
  config,
  trackEvent,
  setUser,
  on,
  off,
  refresh,
  clearCache
} = useGrooveshop()

// Track custom event
trackEvent('view', {
  productId: '123',
  userId: 'user-456'
})

// Set user context
setUser({
  id: 'user-456',
  email: 'user@example.com',
  name: 'John Doe'
})

// Listen to events
on('click', (event) => {
  console.log('Product clicked:', event.productId)
})

// Refresh widgets
await refresh()

// Clear cache
clearCache()
</script>
```

**Available Methods:**

| Method | Description |
|--------|-------------|
| `isReady` | Reactive boolean indicating if widget is loaded |
| `config` | Module configuration |
| `trackEvent(type, data)` | Track custom event |
| `setUser(userData)` | Set user context |
| `on(event, callback)` | Register event listener |
| `off(event, callback)` | Remove event listener |
| `refresh()` | Refresh all widgets |
| `clearCache()` | Clear cached recommendations |

## Usage Examples

### Product Page

```vue
<template>
  <div>
    <div class="product-details">
      <!-- Product info -->
    </div>

    <section class="recommendations">
      <h2>You May Also Like</h2>
      <GrooveshopCarousel
        type="similar"
        :product-id="product.id"
        :count="5"
      />
    </section>

    <section class="recommendations">
      <h2>Complete Your Purchase</h2>
      <GrooveshopGrid
        type="complement"
        :product-id="product.id"
        :count="4"
      />
    </section>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  product: {
    id: string
    name: string
    price: number
  }
}>()
</script>
```

### Homepage

```vue
<template>
  <div>
    <section class="hero">
      <h1>Trending Now</h1>
      <GrooveshopGrid
        type="trending"
        :count="8"
      />
    </section>

    <section v-if="user">
      <h2>Picked For You</h2>
      <GrooveshopCarousel
        type="personalized"
        :user-id="user.id"
        :count="12"
      />
    </section>

    <section>
      <h2>Continue Browsing</h2>
      <GrooveshopCarousel
        type="recently-viewed"
        :count="6"
      />
    </section>
  </div>
</template>

<script setup lang="ts">
const user = useUser() // Your auth composable
</script>
```

### Cart Page

```vue
<template>
  <div>
    <div class="cart-items">
      <!-- Cart items -->
    </div>

    <section class="upsell">
      <h3>Frequently Bought Together</h3>
      <GrooveshopCarousel
        type="bundle"
        :count="4"
        :real-time="true"
      />
    </section>
  </div>
</template>
```

### With Event Tracking

```vue
<template>
  <GrooveshopCarousel
    type="similar"
    :product-id="product.id"
    :count="5"
  />
</template>

<script setup lang="ts">
const { on } = useGrooveshop()

// Track clicks in Google Analytics
on('click', (event) => {
  gtag('event', 'recommendation_click', {
    product_id: event.productId,
    position: event.position
  })
})

// Track impressions
on('impression', (event) => {
  gtag('event', 'recommendation_view', {
    product_ids: event.productIds
  })
})
</script>
```

### Dynamic User Updates

```vue
<script setup lang="ts">
const { setUser, refresh } = useGrooveshop()
const user = useUser()

// Update user context when user logs in
watch(user, async (newUser) => {
  if (newUser) {
    setUser({
      id: newUser.id,
      email: newUser.email,
      name: newUser.name
    })

    // Refresh to show personalized recommendations
    await refresh()
  }
})
</script>
```

## Customization

### Custom Styling

Add custom CSS to override default styles:

```vue
<style>
.gs-recommendations {
  --gs-primary-color: #e74c3c;
  --gs-border-radius: 12px;
  --gs-font-family: 'Inter', sans-serif;
}

.gs-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}
</style>
```

See [Theming Guide](https://docs.grooveshop.com/guides/theming) for all CSS variables.

### Conditional Rendering

```vue
<template>
  <div>
    <!-- Show personalized for logged-in users -->
    <GrooveshopCarousel
      v-if="user"
      type="personalized"
      :user-id="user.id"
      :count="6"
    />

    <!-- Show trending for guests -->
    <GrooveshopCarousel
      v-else
      type="trending"
      :count="6"
    />
  </div>
</template>
```

## TypeScript Support

Full TypeScript support included:

```vue
<script setup lang="ts">
import type { GrooveshopRecommendationsProps } from '@grooveshop/nuxt-recommendations'

const props: GrooveshopRecommendationsProps = {
  type: 'similar',
  productId: '123',
  count: 5
}
</script>
```

The composable is also fully typed:

```ts
const { trackEvent, setUser } = useGrooveshop()

trackEvent('view', { productId: '123' }) // ✅ Typed
setUser({ id: 'user-456', email: 'user@example.com' }) // ✅ Typed
```

## Performance

The module is highly optimized:

- **Lazy Loading**: Images load as user scrolls
- **Prefetching**: Product pages preload on hover
- **Caching**: API responses cached for 5 minutes
- **CDN Delivery**: Widget served from global CDN
- **Small Bundle**: Only 8.8 KB gzipped

### Preconnect

The module automatically adds preconnect links for faster loading:

```html
<link rel="preconnect" href="https://cdn.grooveshop.com">
<link rel="preconnect" href="https://api.grooveshop.com">
```

## Browser Support

- Chrome/Edge: Last 2 versions
- Firefox: Last 2 versions
- Safari: Last 2 versions
- iOS Safari: Last 2 versions
- Android Chrome: Last 2 versions

## Troubleshooting

### Widgets Not Showing?

1. Check API credentials in `nuxt.config.ts`
2. Open browser console for errors
3. Verify network requests in DevTools

### TypeScript Errors?

Run type generation:

```bash
npx nuxi prepare
```

### SSR Issues?

The module automatically handles client-side initialization. Make sure you're not accessing `window` in SSR context.

## Development

```bash
# Install dependencies
npm install

# Develop with playground
npm run dev

# Build module
npm run prepack

# Run tests
npm run test

# Type check
npm run test:types
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE) for details

## Support

- **Documentation**: [docs.grooveshop.com](https://docs.grooveshop.com)
- **Support**: [support.grooveshop.com](https://support.grooveshop.com)
- **Dashboard**: [dashboard.grooveshop.com](https://dashboard.grooveshop.com)
- **GitHub**: [github.com/grooveshop/recommendations](https://github.com/grooveshop/recommendations)

## Related

- [`@grooveshop/recommendations-widget`](https://www.npmjs.com/package/@grooveshop/recommendations-widget) - Vanilla JavaScript widget
- [`@grooveshop/recommendations-react`](https://www.npmjs.com/package/@grooveshop/recommendations-react) - React components
- [`@grooveshop/recommendations-vue`](https://www.npmjs.com/package/@grooveshop/recommendations-vue) - Vue 3 components

---

Made with ❤️ by [GrooveShop](https://grooveshop.com)
