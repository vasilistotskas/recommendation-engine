# Widget API Reference

Complete API reference for the GrooveShop Recommendations widget.

## Table of Contents

- [Initialization](#initialization)
- [Configuration Options](#configuration-options)
- [Widget Attributes](#widget-attributes)
- [Methods](#methods)
- [Events](#events)
- [Types](#types)

---

## Initialization

### `GrooveShopRecommendations.init(config)`

Initialize the widget with your configuration.

**Parameters:**
- `config` (Object) - Configuration object

**Returns:** `void`

**Example:**
```javascript
GrooveShopRecommendations.init({
  apiKey: 'pk_live_abc123',
  tenantId: 'my-store',
  autoTrack: true,
  debug: false
});
```

---

## Configuration Options

### Global Configuration

Configuration passed to `init()` applies to all widgets on the page.

```typescript
interface WidgetConfig {
  // Required
  apiKey: string;           // Your public API key (starts with pk_)
  tenantId: string;         // Your store identifier

  // Optional
  apiUrl?: string;          // API endpoint (default: 'https://api.grooveshop.com')
  autoTrack?: boolean;      // Auto-track clicks/impressions (default: true)
  debug?: boolean;          // Enable console logging (default: false)
  lazyLoad?: boolean;       // Lazy load images (default: true)
  prefetch?: boolean;       // Prefetch on hover (default: true)
  cacheTimeout?: number;    // Cache TTL in ms (default: 300000 = 5 min)
  realtime?: boolean;       // Enable WebSocket updates (default: false)
  wsUrl?: string;           // WebSocket URL (default: wss://api.grooveshop.com/ws)
}
```

### Example: Minimal Configuration

```javascript
GrooveShopRecommendations.init({
  apiKey: 'pk_live_abc123',
  tenantId: 'my-store'
});
```

### Example: Full Configuration

```javascript
GrooveShopRecommendations.init({
  apiKey: 'pk_live_abc123',
  tenantId: 'my-store',
  apiUrl: 'https://api.grooveshop.com',
  autoTrack: true,
  debug: true,
  lazyLoad: true,
  prefetch: true,
  cacheTimeout: 600000,  // 10 minutes
  realtime: true,
  wsUrl: 'wss://api.grooveshop.com/ws'
});
```

---

## Widget Attributes

Attributes added to widget container elements (`<div data-grooveshop-recommendations>`).

### Required Attributes

None! The widget will use intelligent defaults.

### Optional Attributes

```typescript
interface WidgetAttributes {
  'data-type'?: 'similar' | 'trending' | 'bundle' | 'personalized' |
                'complement' | 'recently-viewed' | 'auto';
  'data-product-id'?: string;      // Product ID for context
  'data-user-id'?: string;         // User ID for personalization
  'data-count'?: string;           // Number of products (1-20)
  'data-layout'?: 'carousel' | 'grid' | 'list';
  'data-theme'?: 'light' | 'dark' | 'minimal';
  'data-real-time'?: 'true' | 'false';
}
```

### Recommendation Types

#### `similar`
Show products similar to a specific product.

```html
<div data-grooveshop-recommendations
     data-type="similar"
     data-product-id="123"
     data-count="5">
</div>
```

**Use cases:**
- Product detail pages
- "You may also like" sections
- Cross-sell opportunities

**Auto-detection:** Automatically detects product ID from URL or page meta tags if not specified.

---

#### `trending`
Show currently trending products across your store.

```html
<div data-grooveshop-recommendations
     data-type="trending"
     data-count="8"
     data-layout="grid">
</div>
```

**Use cases:**
- Homepage hero sections
- Category pages
- "Popular now" sections

**Algorithm:** Based on views, clicks, and purchases in last 24 hours.

---

#### `bundle`
Show products that are frequently bought together.

```html
<div data-grooveshop-recommendations
     data-type="bundle"
     data-count="4">
</div>
```

**Use cases:**
- Cart pages
- Checkout process
- "Complete the look" sections

**Auto-detection:** Automatically detects cart contents if available.

---

#### `personalized`
Show AI-powered personalized recommendations for specific users.

```html
<div data-grooveshop-recommendations
     data-type="personalized"
     data-user-id="user-456"
     data-count="6">
</div>
```

**Use cases:**
- Account dashboard
- Email campaigns
- "Picked for you" sections

**Auto-detection:** Uses session ID if user ID not provided.

---

#### `complement`
Show products that complement a specific product.

```html
<div data-grooveshop-recommendations
     data-type="complement"
     data-product-id="123"
     data-count="5">
</div>
```

**Use cases:**
- Product pages
- "Goes well with" sections
- Accessory suggestions

**Example:** Showing phone cases when viewing a phone.

---

#### `recently-viewed`
Show products the user recently viewed.

```html
<div data-grooveshop-recommendations
     data-type="recently-viewed"
     data-count="6">
</div>
```

**Use cases:**
- Homepage for returning visitors
- "Continue browsing" sections
- Exit intent popups

**Storage:** Uses localStorage, works without user ID.

---

#### `auto`
Automatically choose the best recommendation type based on context.

```html
<div data-grooveshop-recommendations
     data-type="auto"
     data-count="5">
</div>
```

**Logic:**
1. If on product page → `similar`
2. If in cart → `bundle`
3. If logged in → `personalized`
4. Otherwise → `trending`

---

### Layout Options

#### `carousel` (default)
Horizontal scrollable carousel with navigation arrows.

```html
<div data-grooveshop-recommendations
     data-layout="carousel">
</div>
```

**Features:**
- Touch/swipe support
- Keyboard navigation
- Responsive breakpoints
- Infinite scroll option

**Best for:**
- Limited vertical space
- Showcasing 5-10 products

---

#### `grid`
Responsive grid layout with equal-height cards.

```html
<div data-grooveshop-recommendations
     data-layout="grid">
</div>
```

**Columns:**
- Mobile: 2 columns
- Tablet: 3 columns
- Desktop: 4 columns

**Best for:**
- Category pages
- Large product sets (8-20 items)

---

#### `list`
Vertical stacked layout.

```html
<div data-grooveshop-recommendations
     data-layout="list">
</div>
```

**Features:**
- Compact design
- Quick scanning
- Detailed product info

**Best for:**
- Sidebars
- Mobile-first designs
- Small sets (3-5 items)

---

### Theme Options

#### `light` (default)
Light background with dark text.

```html
<div data-grooveshop-recommendations
     data-theme="light">
</div>
```

---

#### `dark`
Dark background with light text.

```html
<div data-grooveshop-recommendations
     data-theme="dark">
</div>
```

---

#### `minimal`
Minimal design with no borders or shadows.

```html
<div data-grooveshop-recommendations
     data-theme="minimal">
</div>
```

---

### Count

Number of products to display (1-20).

```html
<div data-grooveshop-recommendations
     data-count="8">
</div>
```

**Default:** `5`

**Recommendations:**
- Carousel: 4-8 items
- Grid: 8-16 items
- List: 3-6 items

---

### Real-time Updates

Enable live social proof and trending updates via WebSocket.

```html
<div data-grooveshop-recommendations
     data-real-time="true">
</div>
```

**Features:**
- "X people viewing this" badges
- Live trending updates
- Stock level changes
- Price updates

**Note:** Requires `realtime: true` in global config.

---

## Methods

### `GrooveShopRecommendations.init(config)`

Initialize the widget (see [Initialization](#initialization)).

---

### `GrooveShopRecommendations.trackEvent(type, data)`

Manually track an event.

**Parameters:**
- `type` (string) - Event type: `'view'`, `'click'`, `'impression'`, `'add_to_cart'`, `'purchase'`
- `data` (Object) - Event data

**Returns:** `Promise<void>`

**Example:**
```javascript
GrooveShopRecommendations.trackEvent('view', {
  productId: '123',
  userId: 'user-456'
});

GrooveShopRecommendations.trackEvent('add_to_cart', {
  productId: '123',
  quantity: 2,
  price: 29.99
});
```

---

### `GrooveShopRecommendations.setUser(userData)`

Set user context for personalization.

**Parameters:**
- `userData` (Object) - User information

**Returns:** `void`

**Example:**
```javascript
GrooveShopRecommendations.setUser({
  id: 'user-456',
  email: 'user@example.com',
  name: 'John Doe'
});
```

---

### `GrooveShopRecommendations.on(event, callback)`

Register event listener.

**Parameters:**
- `event` (string) - Event name: `'click'`, `'impression'`, `'load'`, `'error'`
- `callback` (Function) - Event handler

**Returns:** `Function` (unsubscribe function)

**Example:**
```javascript
const unsubscribe = GrooveShopRecommendations.on('click', (data) => {
  console.log('Product clicked:', data.productId);

  // Send to Google Analytics
  gtag('event', 'recommendation_click', {
    product_id: data.productId,
    position: data.position
  });
});

// Later: unsubscribe()
```

---

### `GrooveShopRecommendations.off(event, callback)`

Remove event listener.

**Parameters:**
- `event` (string) - Event name
- `callback` (Function) - Event handler to remove

**Returns:** `void`

**Example:**
```javascript
function handleClick(data) {
  console.log('Clicked:', data);
}

GrooveShopRecommendations.on('click', handleClick);
GrooveShopRecommendations.off('click', handleClick);
```

---

### `GrooveShopRecommendations.refresh()`

Refresh all widgets on the page.

**Returns:** `Promise<void>`

**Example:**
```javascript
// Refresh after user logs in
document.getElementById('login-form').addEventListener('submit', async () => {
  await login();
  await GrooveShopRecommendations.refresh();
});
```

---

### `GrooveShopRecommendations.clearCache()`

Clear cached recommendations.

**Returns:** `void`

**Example:**
```javascript
// Clear cache when products are updated
GrooveShopRecommendations.clearCache();
GrooveShopRecommendations.refresh();
```

---

## Events

### Event Types

```typescript
type EventType = 'click' | 'impression' | 'load' | 'error';
```

### Event Data Structures

#### Click Event

```typescript
interface ClickEvent {
  type: 'click';
  productId: string;
  position: number;
  sourceType: 'similar' | 'trending' | 'bundle' | etc.;
  sourceProductId?: string;
  timestamp: number;
  metadata?: Record<string, any>;
}
```

**Example:**
```javascript
GrooveShopRecommendations.on('click', (event) => {
  console.log('Product clicked:', event.productId);
  console.log('Position:', event.position);
  console.log('Source:', event.sourceType);
});
```

---

#### Impression Event

```typescript
interface ImpressionEvent {
  type: 'impression';
  productIds: string[];
  sourceType: string;
  sourceProductId?: string;
  timestamp: number;
}
```

**Example:**
```javascript
GrooveShopRecommendations.on('impression', (event) => {
  console.log('Products viewed:', event.productIds);
});
```

---

#### Load Event

```typescript
interface LoadEvent {
  type: 'load';
  productCount: number;
  loadTime: number;  // ms
  sourceType: string;
}
```

**Example:**
```javascript
GrooveShopRecommendations.on('load', (event) => {
  console.log('Widget loaded in', event.loadTime, 'ms');
  console.log('Showing', event.productCount, 'products');
});
```

---

#### Error Event

```typescript
interface ErrorEvent {
  type: 'error';
  error: Error;
  context: string;
}
```

**Example:**
```javascript
GrooveShopRecommendations.on('error', (event) => {
  console.error('Widget error:', event.error.message);
  console.log('Context:', event.context);
});
```

---

## Types

### Product

```typescript
interface Product {
  entity_id: string;
  attributes: {
    name: string;
    price: number;
    image_url: string;
    url: string;
    discount_price?: number;
    currency?: string;
    rating?: number;
    review_count?: number;
    in_stock?: boolean;
    category?: string;
    tags?: string[];
  };
}
```

### Recommendation Response

```typescript
interface RecommendationResponse {
  products: Product[];
  algorithm: string;
  context?: Record<string, any>;
}
```

---

## Advanced Usage

### Custom Product Click Handler

```javascript
GrooveShopRecommendations.on('click', (event) => {
  // Prevent default navigation
  event.preventDefault();

  // Custom logic
  showQuickView(event.productId);

  // Track in analytics
  analytics.track('Recommendation Clicked', {
    product_id: event.productId,
    position: event.position
  });
});
```

### Dynamic User Updates

```javascript
// Update user context when user logs in
document.getElementById('login').addEventListener('click', async () => {
  const user = await loginUser();

  GrooveShopRecommendations.setUser({
    id: user.id,
    email: user.email
  });

  // Refresh to show personalized recommendations
  await GrooveShopRecommendations.refresh();
});
```

### Performance Monitoring

```javascript
GrooveShopRecommendations.on('load', (event) => {
  // Send to performance monitoring
  if (window.performance && window.performance.mark) {
    performance.mark(`recommendations-${event.sourceType}-loaded`);
  }

  // Track slow loads
  if (event.loadTime > 1000) {
    console.warn('Slow widget load:', event.loadTime, 'ms');
  }
});
```

---

## Browser Support

- Chrome/Edge: Last 2 versions
- Firefox: Last 2 versions
- Safari: Last 2 versions
- iOS Safari: Last 2 versions
- Android Chrome: Last 2 versions

## TypeScript Support

Full TypeScript definitions included:

```typescript
import type {
  WidgetConfig,
  WidgetAttributes,
  Product,
  ClickEvent,
  ImpressionEvent
} from '@grooveshop/recommendations-widget';
```

---

## See Also

- [Configuration Guide](../guides/configuration.md)
- [Event Tracking](../guides/event-tracking.md)
- [REST API Reference](./rest-api.md)
- [TypeScript Definitions](./typescript.md)
