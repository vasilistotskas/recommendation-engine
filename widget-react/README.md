# @grooveshop/recommendations-react

React components for GrooveShop product recommendations.

## Installation

```bash
npm install @grooveshop/recommendations-react
```

## Quick Start

### Basic Usage

```tsx
import { RecommendationCarousel } from '@grooveshop/recommendations-react';

function ProductPage({ productId }: { productId: string }) {
  return (
    <RecommendationCarousel
      apiKey="pk_live_abc123"
      tenantId="my-store"
      productId={productId}
      count={5}
    />
  );
}
```

### With Provider (Recommended)

```tsx
import { RecommendationProvider, RecommendationCarousel } from '@grooveshop/recommendations-react';

function App() {
  return (
    <RecommendationProvider
      config={{
        apiKey: 'pk_live_abc123',
        tenantId: 'my-store',
        apiUrl: 'https://api.grooveshop.com',
        debug: false,
      }}
    >
      <ProductPage />
    </RecommendationProvider>
  );
}

function ProductPage() {
  return (
    <div>
      <h1>Similar Products</h1>
      <RecommendationCarousel
        productId="123"
        type="similar"
        count={5}
      />
    </div>
  );
}
```

## Components

### RecommendationCarousel

Horizontal scrollable carousel layout.

```tsx
<RecommendationCarousel
  apiKey="pk_live_abc123"
  tenantId="my-store"
  productId="123"
  count={5}
  type="similar"
  onProductClick={(product) => console.log('Clicked:', product)}
/>
```

### RecommendationGrid

Responsive grid layout.

```tsx
<RecommendationGrid
  apiKey="pk_live_abc123"
  tenantId="my-store"
  type="trending"
  count={8}
/>
```

### RecommendationList

Vertical list layout.

```tsx
<RecommendationList
  apiKey="pk_live_abc123"
  tenantId="my-store"
  productId="123"
  type="complement"
  count={4}
/>
```

## Props

All components accept the following props:

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `apiKey` | string | Yes | - | Your GrooveShop API key |
| `tenantId` | string | Yes | - | Your store identifier |
| `apiUrl` | string | No | - | API endpoint URL |
| `productId` | string | No | - | Source product ID |
| `count` | number | No | 5 | Number of recommendations |
| `type` | RecommendationType | No | 'similar' | Type of recommendations |
| `theme` | 'light' \| 'dark' \| 'minimal' | No | 'light' | Visual theme |
| `realTime` | boolean | No | false | Enable real-time social proof |
| `userId` | string | No | - | User ID for personalization |
| `testId` | string | No | - | A/B test ID |
| `onProductClick` | (product) => void | No | - | Click event handler |
| `onProductImpression` | (products) => void | No | - | Impression event handler |
| `className` | string | No | '' | Additional CSS class |
| `style` | CSSProperties | No | {} | Inline styles |

## Recommendation Types

- `similar` - Products similar to the current product
- `trending` - Trending products
- `bundle` - Frequently bought together
- `personalized` - Personalized for the user
- `complement` - Complementary products
- `recently-viewed` - Recently viewed by user
- `auto` - Auto-detect based on context

## Event Handling

### Product Click

```tsx
<RecommendationCarousel
  {...props}
  onProductClick={(product) => {
    console.log('User clicked product:', product.entity_id);
    // Navigate to product page, add to cart, etc.
  }}
/>
```

### Product Impressions

```tsx
<RecommendationCarousel
  {...props}
  onProductImpression={(products) => {
    console.log('Widget displayed', products.length, 'products');
    // Track impression in your analytics
  }}
/>
```

### Custom Event Listeners

```tsx
import { useRecommendations } from '@grooveshop/recommendations-react';

function MyComponent() {
  const { on } = useRecommendations();

  useEffect(() => {
    // Listen to all recommendation events
    const unsubscribe = on('*', (event) => {
      console.log('Event:', event.type, event);
    });

    return unsubscribe;
  }, [on]);

  return <div>...</div>;
}
```

## Real-time Social Proof

Enable live badges showing product activity:

```tsx
<RecommendationCarousel
  {...props}
  realTime={true}
/>
```

Shows badges like:
- "5 viewing now"
- "3 sold today"
- "2 in carts"

## A/B Testing

Configure A/B tests in the provider:

```tsx
<RecommendationProvider
  config={{
    apiKey: 'pk_live_abc123',
    tenantId: 'my-store',
    abTests: {
      'homepage-layout': {
        variants: [
          { name: 'carousel', weight: 50, config: { layout: 'carousel' } },
          { name: 'grid', weight: 50, config: { layout: 'grid' } },
        ],
      },
    },
  }}
>
  <RecommendationCarousel testId="homepage-layout" />
</RecommendationProvider>
```

## Theming

### Using CSS Variables

```css
:root {
  --gs-primary-color: #007bff;
  --gs-font-family: 'Inter', sans-serif;
  --gs-border-radius: 12px;
}
```

### Custom Styles

```tsx
<RecommendationCarousel
  {...props}
  className="my-custom-class"
  style={{ marginTop: '2rem' }}
/>
```

## TypeScript

Full TypeScript support with type definitions included.

```tsx
import type {
  Product,
  RecommendationType,
  RecommendationProps,
} from '@grooveshop/recommendations-react';
```

## License

MIT
