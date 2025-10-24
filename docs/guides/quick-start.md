# Quick Start Guide

Get GrooveShop Recommendations running on your site in just 5 minutes!

## Prerequisites

- A website (any platform)
- Access to edit HTML
- 5 minutes of your time

## Step 1: Sign Up (1 minute)

1. Go to [dashboard.grooveshop.com](https://dashboard.grooveshop.com)
2. Sign up for a free account
3. Create a new store
4. Copy your credentials:
   - **API Key**: `pk_live_abc123...` (public key)
   - **Tenant ID**: `my-store` (your store identifier)

## Step 2: Add Script Tag (2 minutes)

Add this code to your site's `<head>` section, just before the closing `</head>` tag:

```html
<script src="https://cdn.grooveshop.com/recommendations/v1/widget.js"></script>
<script>
  GrooveShopRecommendations.init({
    apiKey: 'pk_live_your_key_here',      // Replace with your API key
    tenantId: 'your-tenant-id',            // Replace with your tenant ID
    autoTrack: true,                       // Automatically track clicks
    debug: false                           // Set to true for console logs
  });
</script>
```

**Replace**:
- `pk_live_your_key_here` → Your actual API key
- `your-tenant-id` → Your actual tenant ID

## Step 3: Add Widget Container (1 minute)

Add this HTML where you want recommendations to appear:

```html
<div data-grooveshop-recommendations
     data-type="trending"
     data-count="5"
     data-layout="carousel">
</div>
```

## Step 4: Test (1 minute)

1. Save and refresh your page
2. You should see recommendation widgets loading!
3. If not, open browser console (F12) and check for errors

## You're Done! 🎉

Recommendations are now live on your site!

## What's Next?

### Customize Your Widgets

Try different recommendation types:

**Similar Products** (on product pages):
```html
<div data-grooveshop-recommendations
     data-type="similar"
     data-product-id="123"
     data-count="5">
</div>
```

**Trending Products** (homepage):
```html
<div data-grooveshop-recommendations
     data-type="trending"
     data-count="8"
     data-layout="grid">
</div>
```

**Personalized** (for logged-in users):
```html
<div data-grooveshop-recommendations
     data-type="personalized"
     data-user-id="user-456"
     data-count="6">
</div>
```

**Bundle** (cart page):
```html
<div data-grooveshop-recommendations
     data-type="bundle"
     data-count="4">
</div>
```

### Change the Layout

Choose from 3 layouts:

```html
<!-- Carousel (default) - horizontal scrolling -->
data-layout="carousel"

<!-- Grid - responsive columns -->
data-layout="grid"

<!-- List - vertical stacked -->
data-layout="list"
```

### Customize Colors

```html
<style>
  .gs-recommendations {
    --gs-primary-color: #e74c3c;
    --gs-border-radius: 8px;
    --gs-font-family: 'Inter', sans-serif;
  }
</style>
```

### Track Custom Events

```javascript
GrooveShopRecommendations.on('click', (event) => {
  console.log('User clicked product:', event.productId);

  // Send to Google Analytics
  gtag('event', 'recommendation_click', {
    product_id: event.productId,
    position: event.position
  });
});
```

## Platform-Specific Guides

Using a specific platform? We have easier integrations:

### WordPress / WooCommerce
Install our [WordPress plugin](../platforms/wordpress.md) - no code required!

1. Install "GrooveShop Recommendations" from WordPress.org
2. Enter API credentials in Settings
3. Use shortcodes or Gutenberg blocks

### Shopify
Install our [Shopify app](../platforms/shopify.md) - drag and drop!

1. Install from Shopify App Store
2. Enter API credentials
3. Add blocks in theme editor

### React
Use our [React components](./react.md):

```bash
npm install @grooveshop/recommendations-react
```

```jsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

<RecommendationCarousel
  apiKey="pk_live_your_key"
  tenantId="your-tenant-id"
  productId={123}
  count={5}
/>
```

### Vue
Use our [Vue components](./vue.md):

```bash
npm install @grooveshop/recommendations-vue
```

```vue
<template>
  <RecommendationCarousel
    api-key="pk_live_your_key"
    tenant-id="your-tenant-id"
    :product-id="123"
    :count="5"
  />
</template>
```

## Troubleshooting

### Widgets Not Showing?

1. **Check API credentials**: Verify your API key and tenant ID are correct
2. **Open browser console**: Look for error messages (F12 → Console tab)
3. **Check network tab**: Ensure API calls are successful (F12 → Network tab)
4. **Verify container exists**: Make sure `[data-grooveshop-recommendations]` elements are in your HTML

### Common Issues

**Error: "Invalid API key"**
- Double-check your API key starts with `pk_`
- Ensure no extra spaces or quotes
- Verify key is active in your dashboard

**Error: "Product not found"**
- Make sure product ID is correct
- Product must be synced to GrooveShop (happens automatically after first view)

**Widgets showing but no products**
- You might not have enough data yet
- Try `data-type="trending"` which doesn't require specific products
- Check your dashboard to ensure products are synced

**Layout looks broken**
- Ensure widget script is loaded before `init()`
- Check for CSS conflicts with your theme
- Try adding `!important` to custom styles

### Still Need Help?

- Check [FAQ](./faq.md)
- Read [Troubleshooting Guide](./troubleshooting.md)
- Email: support@grooveshop.com
- Dashboard: [dashboard.grooveshop.com](https://dashboard.grooveshop.com)

## Performance Tips

### Lazy Load Widgets

Only load widgets when they're visible:

```javascript
GrooveShopRecommendations.init({
  apiKey: 'pk_live_your_key',
  tenantId: 'your-tenant-id',
  lazyLoad: true  // Load when scrolled into view
});
```

### Prefetch on Hover

Speed up page loads by prefetching on hover:

```javascript
GrooveShopRecommendations.init({
  apiKey: 'pk_live_your_key',
  tenantId: 'your-tenant-id',
  prefetch: true  // Prefetch product pages on hover
});
```

### Preconnect to CDN

Add this to your `<head>` for faster loading:

```html
<link rel="preconnect" href="https://cdn.grooveshop.com">
<link rel="preconnect" href="https://api.grooveshop.com">
```

## Next Steps

Now that you're up and running:

1. ✅ [Configure advanced options](./configuration.md)
2. ✅ [Customize the look and feel](./theming.md)
3. ✅ [Set up event tracking](./event-tracking.md)
4. ✅ [View analytics in your dashboard](https://dashboard.grooveshop.com)
5. ✅ [Explore code examples](../examples/)

## Learn More

- [Configuration Guide](./configuration.md) - All config options
- [API Reference](../api/widget-api.md) - Complete API docs
- [Examples](../examples/) - Code examples and recipes
- [Theming Guide](./theming.md) - Customize appearance

---

**Questions?** We're here to help! Email support@grooveshop.com or check our [FAQ](./faq.md).
