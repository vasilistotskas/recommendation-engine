# Code Recipes

Common patterns and solutions for GrooveShop Recommendations.

## Table of Contents

- [Product Pages](#product-pages)
- [Homepage & Category Pages](#homepage--category-pages)
- [Cart & Checkout](#cart--checkout)
- [User Personalization](#user-personalization)
- [Analytics Integration](#analytics-integration)
- [A/B Testing](#ab-testing)
- [Performance Optimization](#performance-optimization)
- [Error Handling](#error-handling)

---

## Product Pages

### Similar Products Below Description

```html
<div class="product-details">
  <!-- Product info -->
</div>

<div class="similar-products">
  <h2>You May Also Like</h2>
  <div data-grooveshop-recommendations
       data-type="similar"
       data-product-id="{{product.id}}"
       data-count="5"
       data-layout="carousel">
  </div>
</div>
```

### Complementary Products (Accessories)

```html
<h3>Complete Your Purchase</h3>
<div data-grooveshop-recommendations
     data-type="complement"
     data-product-id="{{product.id}}"
     data-count="4"
     data-layout="grid">
</div>
```

### Auto-Detect Product ID

The widget automatically detects product ID from:
1. `data-product-id` attribute
2. URL parameters (`?product_id=123`)
3. Page meta tags (`<meta name="product:id" content="123">`)

```html
<!-- No product ID needed if on product page -->
<div data-grooveshop-recommendations
     data-type="similar"
     data-count="5">
</div>
```

---

## Homepage & Category Pages

### Trending Products Hero Section

```html
<section class="hero">
  <h1>Trending Now</h1>
  <div data-grooveshop-recommendations
       data-type="trending"
       data-count="8"
       data-layout="grid">
  </div>
</section>
```

### Personalized "For You" Section

```html
<section class="personalized">
  <h2>Picked For You</h2>
  <div data-grooveshop-recommendations
       data-type="personalized"
       data-user-id="{{user.id}}"
       data-count="12"
       data-layout="grid">
  </div>
</section>
```

### Recently Viewed for Returning Visitors

```html
<!-- Show only if user has view history -->
<div id="recently-viewed" style="display: none;">
  <h2>Continue Browsing</h2>
  <div data-grooveshop-recommendations
       data-type="recently-viewed"
       data-count="6"
       data-layout="carousel">
  </div>
</div>

<script>
  // Show if user has viewed products
  if (localStorage.getItem('gs_view_history')) {
    document.getElementById('recently-viewed').style.display = 'block';
  }
</script>
```

---

## Cart & Checkout

### Bundle Suggestions in Cart

```html
<div class="cart-items">
  <!-- Cart items list -->
</div>

<div class="cart-upsell">
  <h3>Frequently Bought Together</h3>
  <div data-grooveshop-recommendations
       data-type="bundle"
       data-count="4"
       data-layout="carousel"
       data-real-time="true">
  </div>
</div>
```

### Last Chance Upsell Before Checkout

```html
<div class="checkout-upsell">
  <h4>Before You Go...</h4>
  <div data-grooveshop-recommendations
       data-type="complement"
       data-count="3"
       data-layout="list">
  </div>
</div>

<button class="checkout-btn">Proceed to Checkout</button>
```

---

## User Personalization

### Set User Context on Login

```javascript
// When user logs in
document.addEventListener('userLogin', (event) => {
  GrooveShopRecommendations.setUser({
    id: event.detail.userId,
    email: event.detail.email,
    name: event.detail.name
  });

  // Refresh widgets to show personalized recommendations
  GrooveShopRecommendations.refresh();
});
```

### Dynamic User Updates

```javascript
// Update user context and refresh
function updateUserPreferences(preferences) {
  GrooveShopRecommendations.setUser({
    id: currentUser.id,
    preferences: preferences
  });

  GrooveShopRecommendations.refresh();
}
```

---

## Analytics Integration

### Google Analytics 4

```javascript
// Track recommendation clicks in GA4
GrooveShopRecommendations.on('click', (event) => {
  gtag('event', 'select_item', {
    item_list_id: event.sourceType,
    item_list_name: 'Recommendations',
    items: [{
      item_id: event.productId,
      item_name: event.metadata?.name,
      price: event.metadata?.price,
      index: event.position
    }]
  });
});

// Track impressions
GrooveShopRecommendations.on('impression', (event) => {
  gtag('event', 'view_item_list', {
    item_list_id: event.sourceType,
    item_list_name: 'Recommendations',
    items: event.productIds.map((id, index) => ({
      item_id: id,
      index: index
    }))
  });
});
```

### Segment

```javascript
// Track with Segment
GrooveShopRecommendations.on('click', (event) => {
  analytics.track('Product Clicked', {
    product_id: event.productId,
    source: 'recommendations',
    position: event.position,
    recommendation_type: event.sourceType
  });
});
```

### Facebook Pixel

```javascript
// Track with Facebook Pixel
GrooveShopRecommendations.on('click', (event) => {
  fbq('track', 'ViewContent', {
    content_ids: [event.productId],
    content_type: 'product',
    source: 'recommendation'
  });
});
```

### Custom Analytics Platform

```javascript
// Send to custom endpoint
GrooveShopRecommendations.on('click', async (event) => {
  await fetch('/api/analytics/recommendation-click', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      product_id: event.productId,
      position: event.position,
      timestamp: event.timestamp,
      user_id: getCurrentUserId()
    })
  });
});
```

---

## A/B Testing

### Test Different Layouts

```javascript
// Assign user to variant
const variant = getUserVariant('homepage_recommendations'); // 'carousel' or 'grid'

const container = document.getElementById('recommendations');
container.setAttribute('data-layout', variant);

// Track which variant user saw
GrooveShopRecommendations.on('impression', (event) => {
  analytics.track('Recommendation Viewed', {
    variant: variant,
    products: event.productIds
  });
});

// Track conversion by variant
GrooveShopRecommendations.on('click', (event) => {
  analytics.track('Recommendation Clicked', {
    variant: variant,
    product_id: event.productId
  });
});
```

### Test Widget Placement

```javascript
// Test: Above fold vs. below fold
const placement = Math.random() < 0.5 ? 'above-fold' : 'below-fold';

if (placement === 'above-fold') {
  document.querySelector('.hero').insertAdjacentHTML('afterend',
    '<div data-grooveshop-recommendations data-type="trending"></div>'
  );
} else {
  document.querySelector('.footer').insertAdjacentHTML('beforebegin',
    '<div data-grooveshop-recommendations data-type="trending"></div>'
  );
}

// Initialize after inserting
GrooveShopRecommendations.init({
  apiKey: 'pk_live_key',
  tenantId: 'my-store'
});

// Track performance by placement
GrooveShopRecommendations.on('click', (event) => {
  analytics.track('Recommendation Clicked', {
    placement: placement
  });
});
```

---

## Performance Optimization

### Lazy Load Widgets

```javascript
// Load widgets only when scrolled into view
const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      // Initialize widget when visible
      GrooveShopRecommendations.init({
        apiKey: 'pk_live_key',
        tenantId: 'my-store',
        lazyLoad: true
      });

      observer.unobserve(entry.target);
    }
  });
});

document.querySelectorAll('[data-grooveshop-recommendations]').forEach(el => {
  observer.observe(el);
});
```

### Preconnect to CDN

```html
<!-- Add to <head> for faster loading -->
<link rel="preconnect" href="https://cdn.grooveshop.com">
<link rel="preconnect" href="https://api.grooveshop.com">
<link rel="dns-prefetch" href="https://cdn.grooveshop.com">
<link rel="dns-prefetch" href="https://api.grooveshop.com">
```

### Prefetch Product Pages

```javascript
// Enable prefetch on hover
GrooveShopRecommendations.init({
  apiKey: 'pk_live_key',
  tenantId: 'my-store',
  prefetch: true  // Prefetch product pages on hover
});
```

### Cache Recommendations

```javascript
// Increase cache timeout for better performance
GrooveShopRecommendations.init({
  apiKey: 'pk_live_key',
  tenantId: 'my-store',
  cacheTimeout: 600000  // 10 minutes
});
```

---

## Error Handling

### Graceful Fallbacks

```javascript
GrooveShopRecommendations.on('error', (event) => {
  console.error('Widget error:', event.error);

  // Hide widget on error
  document.querySelectorAll('[data-grooveshop-recommendations]').forEach(el => {
    el.style.display = 'none';
  });

  // Show fallback content
  document.getElementById('fallback-products').style.display = 'block';
});
```

### Retry on Error

```javascript
let retryCount = 0;
const maxRetries = 3;

GrooveShopRecommendations.on('error', async (event) => {
  if (retryCount < maxRetries) {
    retryCount++;
    console.log(`Retrying... (${retryCount}/${maxRetries})`);

    // Wait 1 second and retry
    await new Promise(resolve => setTimeout(resolve, 1000));
    GrooveShopRecommendations.refresh();
  } else {
    console.error('Max retries reached');
  }
});
```

### Error Reporting

```javascript
// Send errors to monitoring service
GrooveShopRecommendations.on('error', (event) => {
  // Report to Sentry
  if (window.Sentry) {
    Sentry.captureException(event.error, {
      tags: {
        component: 'recommendations',
        context: event.context
      }
    });
  }

  // Or custom error tracking
  fetch('/api/errors', {
    method: 'POST',
    body: JSON.stringify({
      error: event.error.message,
      stack: event.error.stack,
      context: event.context
    })
  });
});
```

---

## Advanced Patterns

### Progressive Enhancement

```html
<!-- Show manual product list initially -->
<div class="product-recommendations">
  <div class="product-card">Product 1</div>
  <div class="product-card">Product 2</div>
  <div class="product-card">Product 3</div>
</div>

<script>
  // Replace with widget when loaded
  document.querySelector('.product-recommendations').outerHTML =
    '<div data-grooveshop-recommendations data-type="trending"></div>';

  GrooveShopRecommendations.init({
    apiKey: 'pk_live_key',
    tenantId: 'my-store'
  });
</script>
```

### Conditional Rendering

```javascript
// Show different widgets based on conditions
const user = getCurrentUser();

if (user && user.isLoggedIn) {
  // Personalized for logged-in users
  document.getElementById('recommendations').innerHTML =
    '<div data-grooveshop-recommendations data-type="personalized" data-user-id="' + user.id + '"></div>';
} else {
  // Trending for guests
  document.getElementById('recommendations').innerHTML =
    '<div data-grooveshop-recommendations data-type="trending"></div>';
}

GrooveShopRecommendations.init({
  apiKey: 'pk_live_key',
  tenantId: 'my-store'
});
```

### Multiple Widgets on Same Page

```html
<!-- Similar products -->
<section>
  <h2>Similar Products</h2>
  <div data-grooveshop-recommendations
       data-type="similar"
       data-product-id="123">
  </div>
</section>

<!-- Trending products -->
<section>
  <h2>Trending Now</h2>
  <div data-grooveshop-recommendations
       data-type="trending"
       data-layout="grid">
  </div>
</section>

<!-- Recently viewed -->
<section>
  <h2>Recently Viewed</h2>
  <div data-grooveshop-recommendations
       data-type="recently-viewed">
  </div>
</section>
```

### Custom Click Handler

```javascript
// Prevent default navigation and show quick view instead
GrooveShopRecommendations.on('click', (event) => {
  event.preventDefault();

  // Show product quick view modal
  showProductQuickView(event.productId);

  // Track in analytics
  analytics.track('Quick View Opened', {
    product_id: event.productId,
    source: 'recommendations'
  });
});
```

### Real-time Social Proof

```html
<div data-grooveshop-recommendations
     data-type="trending"
     data-real-time="true"
     data-count="8">
</div>

<script>
  GrooveShopRecommendations.init({
    apiKey: 'pk_live_key',
    tenantId: 'my-store',
    realtime: true,
    wsUrl: 'wss://api.grooveshop.com/ws'
  });
</script>
```

---

## Platform-Specific Recipes

### Shopify

```liquid
<!-- Product page -->
{% if product %}
  <div data-grooveshop-recommendations
       data-type="similar"
       data-product-id="{{ product.id }}"
       data-count="5">
  </div>
{% endif %}

<!-- Collection page -->
<div data-grooveshop-recommendations
     data-type="trending"
     data-count="8"
     data-layout="grid">
</div>
```

### WordPress/WooCommerce

```php
<!-- Product page -->
<?php echo do_shortcode('[grooveshop_recommendations type="similar" count="5"]'); ?>

<!-- Homepage -->
<?php echo do_shortcode('[grooveshop_recommendations type="trending" count="8" layout="grid"]'); ?>
```

### Next.js

```jsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

export default function ProductPage({ product }) {
  return (
    <div>
      <h2>Similar Products</h2>
      <RecommendationCarousel
        type="similar"
        productId={product.id}
        count={5}
      />
    </div>
  );
}
```

---

## See Also

- [Quick Start Guide](../guides/quick-start.md)
- [Widget API Reference](../api/widget-api.md)
- [React Guide](../guides/react.md)
- [Vue Guide](../guides/vue.md)
