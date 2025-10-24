# React Integration Guide

Complete guide to using GrooveShop Recommendations with React.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Components](#components)
- [Props](#props)
- [Events](#events)
- [Context Provider](#context-provider)
- [TypeScript](#typescript)
- [Examples](#examples)
- [Best Practices](#best-practices)

---

## Installation

### NPM

```bash
npm install @grooveshop/recommendations-react
```

### Yarn

```bash
yarn add @grooveshop/recommendations-react
```

### Peer Dependencies

The package requires React 18+:

```json
{
  "react": ">=18.0.0",
  "react-dom": ">=18.0.0"
}
```

---

## Quick Start

### Basic Usage

```jsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

function App() {
  return (
    <RecommendationCarousel
      apiKey="pk_live_your_key"
      tenantId="your-tenant-id"
      type="trending"
      count={5}
    />
  );
}
```

### With Provider (Recommended)

```jsx
import { RecommendationProvider, RecommendationCarousel } from '@grooveshop/recommendations-react';

function App() {
  return (
    <RecommendationProvider
      apiKey="pk_live_your_key"
      tenantId="your-tenant-id"
    >
      <RecommendationCarousel
        type="trending"
        count={5}
      />
    </RecommendationProvider>
  );
}
```

---

## Components

### `<RecommendationCarousel />`

Horizontal scrollable carousel layout.

```jsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

<RecommendationCarousel
  apiKey="pk_live_your_key"
  tenantId="your-tenant-id"
  productId="123"
  type="similar"
  count={5}
  theme="light"
  onProductClick={(product) => console.log('Clicked:', product)}
/>
```

---

### `<RecommendationGrid />`

Responsive grid layout.

```jsx
import { RecommendationGrid } from '@grooveshop/recommendations-react';

<RecommendationGrid
  apiKey="pk_live_your_key"
  tenantId="your-tenant-id"
  type="trending"
  count={8}
  theme="dark"
/>
```

---

### `<RecommendationList />`

Vertical list layout.

```jsx
import { RecommendationList } from '@grooveshop/recommendations-react';

<RecommendationList
  apiKey="pk_live_your_key"
  tenantId="your-tenant-id"
  type="recently-viewed"
  count={6}
/>
```

---

### `<Recommendations />`

Base component with custom layout.

```jsx
import { Recommendations } from '@grooveshop/recommendations-react';

<Recommendations
  apiKey="pk_live_your_key"
  tenantId="your-tenant-id"
  type="personalized"
  layout="carousel"  // or 'grid', 'list'
  count={5}
/>
```

---

## Props

### Common Props

All components accept these props:

```typescript
interface RecommendationProps {
  // Required (if not using Provider)
  apiKey?: string;
  tenantId?: string;

  // Widget configuration
  productId?: string;
  userId?: string;
  type?: 'similar' | 'trending' | 'bundle' | 'personalized' |
         'complement' | 'recently-viewed' | 'auto';
  count?: number;
  layout?: 'carousel' | 'grid' | 'list';
  theme?: 'light' | 'dark' | 'minimal';
  realTime?: boolean;

  // Event handlers
  onProductClick?: (product: Product, event: ClickEvent) => void;
  onImpression?: (products: Product[]) => void;
  onLoad?: (data: LoadEvent) => void;
  onError?: (error: Error) => void;

  // Advanced
  className?: string;
  style?: React.CSSProperties;
}
```

### Prop Details

#### `apiKey` (string)

Your public API key.

```jsx
<RecommendationCarousel apiKey="pk_live_abc123" />
```

**Note:** Not required if using `RecommendationProvider`.

---

#### `tenantId` (string)

Your store identifier.

```jsx
<RecommendationCarousel tenantId="my-store" />
```

**Note:** Not required if using `RecommendationProvider`.

---

#### `productId` (string, optional)

Product ID for contextual recommendations (similar, complement).

```jsx
<RecommendationCarousel
  type="similar"
  productId={product.id}
/>
```

**Auto-detection:** If not provided, attempts to detect from URL or page context.

---

#### `userId` (string, optional)

User ID for personalized recommendations.

```jsx
<RecommendationCarousel
  type="personalized"
  userId={user.id}
/>
```

---

#### `type` (string, optional)

Recommendation type. Default: `'similar'`

```jsx
<RecommendationCarousel type="trending" />
```

**Options:**
- `similar` - Similar products
- `trending` - Trending products
- `bundle` - Frequently bought together
- `personalized` - AI-powered personalized
- `complement` - Complementary products
- `recently-viewed` - Recently viewed
- `auto` - Automatically choose best type

---

#### `count` (number, optional)

Number of products to display. Default: `5`

```jsx
<RecommendationCarousel count={8} />
```

**Range:** 1-20

---

#### `layout` (string, optional)

Layout type. Default: varies by component

```jsx
<Recommendations layout="grid" />
```

**Options:**
- `carousel` - Horizontal scrolling
- `grid` - Responsive grid
- `list` - Vertical list

---

#### `theme` (string, optional)

Visual theme. Default: `'light'`

```jsx
<RecommendationCarousel theme="dark" />
```

**Options:**
- `light` - Light background
- `dark` - Dark background
- `minimal` - Borderless, minimal

---

#### `realTime` (boolean, optional)

Enable real-time updates via WebSocket. Default: `false`

```jsx
<RecommendationCarousel realTime />
```

**Features:**
- Live "X people viewing" badges
- Real-time trending updates
- Stock level changes

---

## Events

### `onProductClick`

Fired when user clicks a product.

```jsx
<RecommendationCarousel
  onProductClick={(product, event) => {
    console.log('Clicked product:', product.entity_id);
    console.log('Position:', event.position);

    // Navigate to product page
    router.push(`/products/${product.entity_id}`);

    // Track in analytics
    analytics.track('Recommendation Clicked', {
      product_id: product.entity_id,
      product_name: product.attributes.name,
      position: event.position
    });
  }}
/>
```

**Parameters:**
- `product` (Product) - Product data
- `event` (ClickEvent) - Event metadata

---

### `onImpression`

Fired when products are displayed.

```jsx
<RecommendationCarousel
  onImpression={(products) => {
    console.log('Viewed products:', products.map(p => p.entity_id));

    // Track impressions in analytics
    analytics.track('Recommendations Viewed', {
      product_ids: products.map(p => p.entity_id),
      count: products.length
    });
  }}
/>
```

**Parameters:**
- `products` (Product[]) - Array of displayed products

---

### `onLoad`

Fired when widget finishes loading.

```jsx
<RecommendationCarousel
  onLoad={(data) => {
    console.log('Widget loaded in', data.loadTime, 'ms');
    console.log('Showing', data.productCount, 'products');
  }}
/>
```

**Parameters:**
- `data` (LoadEvent) - Load metadata

---

### `onError`

Fired when an error occurs.

```jsx
<RecommendationCarousel
  onError={(error) => {
    console.error('Widget error:', error.message);

    // Report to error tracking
    Sentry.captureException(error);
  }}
/>
```

**Parameters:**
- `error` (Error) - Error object

---

## Context Provider

Use `RecommendationProvider` to share configuration across multiple widgets.

### Basic Provider

```jsx
import { RecommendationProvider, RecommendationCarousel, RecommendationGrid } from '@grooveshop/recommendations-react';

function App() {
  return (
    <RecommendationProvider
      apiKey="pk_live_your_key"
      tenantId="your-tenant-id"
    >
      {/* API key/tenant ID not needed in children */}
      <RecommendationCarousel type="similar" productId="123" />
      <RecommendationGrid type="trending" count={8} />
    </RecommendationProvider>
  );
}
```

### Provider with Global Config

```jsx
<RecommendationProvider
  apiKey="pk_live_your_key"
  tenantId="your-tenant-id"
  config={{
    autoTrack: true,
    debug: false,
    lazyLoad: true,
    prefetch: true,
    realtime: true
  }}
>
  <App />
</RecommendationProvider>
```

### Provider Props

```typescript
interface RecommendationProviderProps {
  apiKey: string;
  tenantId: string;
  config?: {
    apiUrl?: string;
    autoTrack?: boolean;
    debug?: boolean;
    lazyLoad?: boolean;
    prefetch?: boolean;
    cacheTimeout?: number;
    realtime?: boolean;
    wsUrl?: string;
  };
  children: React.ReactNode;
}
```

---

## TypeScript

Full TypeScript support included.

### Import Types

```typescript
import type {
  Product,
  ClickEvent,
  ImpressionEvent,
  LoadEvent,
  RecommendationProps
} from '@grooveshop/recommendations-react';
```

### Product Type

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

### Event Types

```typescript
interface ClickEvent {
  type: 'click';
  productId: string;
  position: number;
  sourceType: string;
  sourceProductId?: string;
  timestamp: number;
  metadata?: Record<string, any>;
}

interface ImpressionEvent {
  type: 'impression';
  productIds: string[];
  sourceType: string;
  timestamp: number;
}

interface LoadEvent {
  type: 'load';
  productCount: number;
  loadTime: number;
  sourceType: string;
}
```

### Typed Component

```typescript
import { RecommendationCarousel, Product, ClickEvent } from '@grooveshop/recommendations-react';

function ProductPage() {
  const handleClick = (product: Product, event: ClickEvent): void => {
    console.log('Clicked:', product.entity_id);
  };

  return (
    <RecommendationCarousel
      apiKey="pk_live_key"
      tenantId="my-store"
      type="similar"
      onProductClick={handleClick}
    />
  );
}
```

---

## Examples

### Example 1: Product Page

```jsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';
import { useRouter } from 'next/router';

function ProductPage({ product }) {
  const router = useRouter();

  return (
    <div>
      {/* Product details... */}

      <h2>You May Also Like</h2>
      <RecommendationCarousel
        apiKey="pk_live_your_key"
        tenantId="your-tenant-id"
        type="similar"
        productId={product.id}
        count={5}
        onProductClick={(product) => {
          router.push(`/products/${product.entity_id}`);
        }}
      />
    </div>
  );
}
```

### Example 2: Homepage

```jsx
import { RecommendationGrid } from '@grooveshop/recommendations-react';

function Homepage() {
  return (
    <div>
      <h1>Trending Now</h1>
      <RecommendationGrid
        apiKey="pk_live_your_key"
        tenantId="your-tenant-id"
        type="trending"
        count={8}
        theme="light"
      />
    </div>
  );
}
```

### Example 3: Cart Page

```jsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

function CartPage({ cartItems }) {
  return (
    <div>
      {/* Cart items... */}

      <h2>Complete Your Purchase</h2>
      <RecommendationCarousel
        apiKey="pk_live_your_key"
        tenantId="your-tenant-id"
        type="bundle"
        count={4}
        realTime
      />
    </div>
  );
}
```

### Example 4: Personalized Dashboard

```jsx
import { RecommendationGrid } from '@grooveshop/recommendations-react';
import { useAuth } from './hooks/useAuth';

function Dashboard() {
  const { user } = useAuth();

  return (
    <div>
      <h1>Picked For You</h1>
      <RecommendationGrid
        apiKey="pk_live_your_key"
        tenantId="your-tenant-id"
        type="personalized"
        userId={user.id}
        count={12}
      />
    </div>
  );
}
```

### Example 5: With Analytics

```jsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';
import { useAnalytics } from './hooks/useAnalytics';

function ProductPage() {
  const analytics = useAnalytics();

  return (
    <RecommendationCarousel
      apiKey="pk_live_your_key"
      tenantId="your-tenant-id"
      type="similar"
      onProductClick={(product, event) => {
        // Track click
        analytics.track('Recommendation Clicked', {
          product_id: product.entity_id,
          product_name: product.attributes.name,
          price: product.attributes.price,
          position: event.position
        });
      }}
      onImpression={(products) => {
        // Track impression
        analytics.track('Recommendations Viewed', {
          product_ids: products.map(p => p.entity_id),
          count: products.length
        });
      }}
    />
  );
}
```

### Example 6: With Error Handling

```jsx
import { useState } from 'react';
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

function ProductPage() {
  const [error, setError] = useState(null);

  return (
    <div>
      {error && (
        <div className="error-message">
          Failed to load recommendations: {error.message}
        </div>
      )}

      <RecommendationCarousel
        apiKey="pk_live_your_key"
        tenantId="your-tenant-id"
        type="similar"
        onError={(err) => {
          setError(err);
          console.error('Widget error:', err);
        }}
      />
    </div>
  );
}
```

### Example 7: Next.js App Router

```jsx
'use client';

import { RecommendationProvider, RecommendationCarousel } from '@grooveshop/recommendations-react';

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        <RecommendationProvider
          apiKey={process.env.NEXT_PUBLIC_GROOVESHOP_API_KEY}
          tenantId={process.env.NEXT_PUBLIC_GROOVESHOP_TENANT_ID}
        >
          {children}
        </RecommendationProvider>
      </body>
    </html>
  );
}
```

---

## Best Practices

### 1. Use Provider for Multiple Widgets

```jsx
// Good: Share config across widgets
<RecommendationProvider apiKey="..." tenantId="...">
  <RecommendationCarousel type="similar" />
  <RecommendationGrid type="trending" />
</RecommendationProvider>

// Bad: Repeat config in each widget
<>
  <RecommendationCarousel apiKey="..." tenantId="..." />
  <RecommendationGrid apiKey="..." tenantId="..." />
</>
```

### 2. Store API Keys Securely

```jsx
// Good: Use environment variables
<RecommendationProvider
  apiKey={process.env.NEXT_PUBLIC_GROOVESHOP_API_KEY}
  tenantId={process.env.NEXT_PUBLIC_GROOVESHOP_TENANT_ID}
/>

// Bad: Hardcode keys
<RecommendationProvider
  apiKey="pk_live_abc123..."
  tenantId="my-store"
/>
```

### 3. Handle User Changes

```jsx
import { useEffect } from 'react';
import { RecommendationGrid } from '@grooveshop/recommendations-react';

function Dashboard({ user }) {
  useEffect(() => {
    // Refresh when user changes
    if (user) {
      // Widget will auto-refresh with new userId
    }
  }, [user?.id]);

  return (
    <RecommendationGrid
      type="personalized"
      userId={user?.id}
      count={12}
    />
  );
}
```

### 4. Memoize Event Handlers

```jsx
import { useCallback } from 'react';
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

function ProductPage() {
  const handleClick = useCallback((product, event) => {
    console.log('Clicked:', product.entity_id);
  }, []);

  return (
    <RecommendationCarousel
      type="similar"
      onProductClick={handleClick}
    />
  );
}
```

### 5. Lazy Load Widgets

```jsx
import { lazy, Suspense } from 'react';

const RecommendationCarousel = lazy(() =>
  import('@grooveshop/recommendations-react').then(m => ({
    default: m.RecommendationCarousel
  }))
);

function ProductPage() {
  return (
    <Suspense fallback={<div>Loading recommendations...</div>}>
      <RecommendationCarousel type="similar" />
    </Suspense>
  );
}
```

---

## Troubleshooting

### Widgets Not Rendering?

1. Ensure API key and tenant ID are correct
2. Check browser console for errors
3. Verify React version is 18+

### TypeScript Errors?

```bash
# Install type definitions
npm install --save-dev @types/react @types/react-dom
```

### Server-Side Rendering Issues?

```jsx
// Use dynamic import to disable SSR
import dynamic from 'next/dynamic';

const RecommendationCarousel = dynamic(
  () => import('@grooveshop/recommendations-react').then(m => m.RecommendationCarousel),
  { ssr: false }
);
```

---

## See Also

- [Vue Integration Guide](./vue.md)
- [Widget API Reference](../api/widget-api.md)
- [Examples & Recipes](../examples/recipes.md)
- [TypeScript Definitions](../api/typescript.md)
