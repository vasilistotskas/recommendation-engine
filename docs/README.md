# GrooveShop Recommendations - Documentation

Welcome to the GrooveShop Recommendations documentation! This guide will help you integrate AI-powered product recommendations into your ecommerce site in just 5 minutes.

## 📚 Documentation Sections

### Getting Started
- [Quick Start Guide](./guides/quick-start.md) - Get up and running in 5 minutes
- [Installation](./guides/installation.md) - Detailed installation instructions
- [Configuration](./guides/configuration.md) - All configuration options explained

### Integration Guides
- [Vanilla JavaScript](./guides/vanilla-js.md) - Pure JavaScript integration
- [React](./guides/react.md) - React component library
- [Vue](./guides/vue.md) - Vue 3 component library
- [WordPress](./platforms/wordpress.md) - WordPress plugin guide
- [Shopify](./platforms/shopify.md) - Shopify app guide

### Customization
- [Theming Guide](./guides/theming.md) - Customize colors, fonts, and styles
- [Layout Options](./guides/layouts.md) - Carousel, grid, and list layouts
- [Custom Templates](./guides/custom-templates.md) - Build your own templates
- [Event Tracking](./guides/event-tracking.md) - Track clicks and conversions

### API Reference
- [Widget API](./api/widget-api.md) - Complete widget API reference
- [REST API](./api/rest-api.md) - Backend API endpoints
- [Events](./api/events.md) - Event types and callbacks
- [TypeScript Definitions](./api/typescript.md) - Type definitions

### Examples & Recipes
- [Basic Examples](./examples/basic.md) - Simple use cases
- [Advanced Examples](./examples/advanced.md) - Complex scenarios
- [Code Recipes](./examples/recipes.md) - Common patterns and solutions
- [Live Demos](./examples/demos.md) - Interactive CodePen demos

### Troubleshooting
- [Common Issues](./guides/troubleshooting.md) - Solutions to common problems
- [FAQ](./guides/faq.md) - Frequently asked questions
- [Performance](./guides/performance.md) - Optimization tips
- [Browser Support](./guides/browser-support.md) - Compatibility information

## 🚀 Quick Start

### 1. Get API Credentials

Sign up at [dashboard.grooveshop.com](https://dashboard.grooveshop.com) to get your:
- **API Key** (starts with `pk_`)
- **Tenant ID** (your store identifier)

### 2. Add Script Tag

Add this to your HTML `<head>`:

```html
<script src="https://cdn.grooveshop.com/recommendations/v1/widget.js"></script>
<script>
  GrooveShopRecommendations.init({
    apiKey: 'pk_live_your_key_here',
    tenantId: 'your-tenant-id'
  });
</script>
```

### 3. Add Widget Container

Add this where you want recommendations to appear:

```html
<div data-grooveshop-recommendations
     data-type="trending"
     data-count="5"
     data-layout="carousel">
</div>
```

That's it! 🎉 Recommendations will appear automatically.

## 📖 Popular Use Cases

### Similar Products on Product Pages

```html
<div data-grooveshop-recommendations
     data-type="similar"
     data-product-id="123"
     data-count="5"
     data-layout="carousel">
</div>
```

### Trending Products on Homepage

```html
<div data-grooveshop-recommendations
     data-type="trending"
     data-count="8"
     data-layout="grid">
</div>
```

### Personalized Recommendations

```html
<div data-grooveshop-recommendations
     data-type="personalized"
     data-user-id="user-456"
     data-count="6"
     data-layout="list">
</div>
```

### Bundle Suggestions in Cart

```html
<div data-grooveshop-recommendations
     data-type="bundle"
     data-count="4"
     data-layout="carousel">
</div>
```

## 🎨 Customization Example

```html
<style>
  /* Override widget colors */
  .gs-recommendations {
    --gs-primary-color: #e74c3c;
    --gs-border-radius: 12px;
    --gs-font-family: 'Poppins', sans-serif;
  }
</style>
```

## 📱 Framework Examples

### React

```jsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

function ProductPage() {
  return (
    <RecommendationCarousel
      apiKey="pk_live_your_key"
      tenantId="your-tenant-id"
      productId="123"
      count={5}
      onProductClick={(product) => console.log('Clicked:', product)}
    />
  );
}
```

### Vue

```vue
<template>
  <RecommendationCarousel
    api-key="pk_live_your_key"
    tenant-id="your-tenant-id"
    :product-id="123"
    :count="5"
    @product-click="handleClick"
  />
</template>

<script setup>
import { RecommendationCarousel } from '@grooveshop/recommendations-vue';

const handleClick = (product) => {
  console.log('Clicked:', product);
};
</script>
```

## 🔧 Advanced Features

- **Real-time Social Proof**: Show live "X people viewing this"
- **A/B Testing**: Test different layouts and measure impact
- **Analytics Integration**: Works with GA4, Segment, Facebook Pixel
- **Lazy Loading**: Images load as user scrolls
- **Prefetching**: Hover to preload product pages
- **Custom Events**: Hook into clicks, impressions, and more

## 🌟 Features

- ✅ **Lightweight**: 8.8 KB gzipped
- ✅ **Fast**: < 100ms time to interactive
- ✅ **Responsive**: Mobile-first design
- ✅ **Accessible**: WCAG AA compliant
- ✅ **Themeable**: CSS custom properties
- ✅ **Framework Agnostic**: Works with any tech stack
- ✅ **TypeScript**: Full type definitions included

## 📦 Packages

| Package | Size | Description |
|---------|------|-------------|
| `@grooveshop/recommendations-widget` | 8.8 KB | Vanilla JavaScript widget |
| `@grooveshop/recommendations-react` | 3.58 KB | React components |
| `@grooveshop/recommendations-vue` | 1.24 KB | Vue 3 components |

## 🛠️ Platform Integrations

- **WordPress**: Install plugin from WordPress.org
- **Shopify**: Install app from Shopify App Store
- **WooCommerce**: Included in WordPress plugin
- **Custom**: Use vanilla widget on any platform

## 📞 Support

- **Documentation**: [docs.grooveshop.com](https://docs.grooveshop.com)
- **Support**: [support.grooveshop.com](https://support.grooveshop.com)
- **Dashboard**: [dashboard.grooveshop.com](https://dashboard.grooveshop.com)
- **GitHub**: [github.com/grooveshop/recommendations](https://github.com/grooveshop/recommendations)
- **Email**: support@grooveshop.com

## 📄 License

MIT License - see [LICENSE](../LICENSE) for details

## 🗺️ What's Next?

1. Read the [Quick Start Guide](./guides/quick-start.md)
2. Explore [Examples](./examples/)
3. Check [API Reference](./api/)
4. Join our community on Discord

---

**Ready to boost your conversions?** Let's get started! 🚀
