# @grooveshop/recommendations-vue

Vue 3 components for GrooveShop product recommendations.

## Installation

```bash
npm install @grooveshop/recommendations-vue
```

## Quick Start

### Basic Usage

```vue
<script setup lang="ts">
import { RecommendationCarousel } from '@grooveshop/recommendations-vue';

const productId = '123';
</script>

<template>
  <RecommendationCarousel
    api-key="pk_live_abc123"
    tenant-id="my-store"
    :product-id="productId"
    :count="5"
  />
</template>
```

### With Composable

```vue
<script setup lang="ts">
import { useRecommendations, RecommendationCarousel } from '@grooveshop/recommendations-vue';

const { on, isInitialized } = useRecommendations({
  apiKey: 'pk_live_abc123',
  tenantId: 'my-store',
  debug: false,
});

// Listen to events
const unsubscribe = on('click', (event) => {
  console.log('Product clicked:', event.productId);
});
</script>

<template>
  <RecommendationCarousel
    product-id="123"
    type="similar"
    :count="5"
  />
</template>
```

## Components

### RecommendationCarousel

Horizontal scrollable carousel layout.

```vue
<template>
  <RecommendationCarousel
    api-key="pk_live_abc123"
    tenant-id="my-store"
    product-id="123"
    :count="5"
    type="similar"
    @product-click="handleClick"
  />
</template>

<script setup>
function handleClick(product) {
  console.log('Clicked:', product);
}
</script>
```

### RecommendationGrid

Responsive grid layout.

```vue
<template>
  <RecommendationGrid
    api-key="pk_live_abc123"
    tenant-id="my-store"
    type="trending"
    :count="8"
  />
</template>
```

### RecommendationList

Vertical list layout.

```vue
<template>
  <RecommendationList
    api-key="pk_live_abc123"
    tenant-id="my-store"
    product-id="123"
    type="complement"
    :count="4"
  />
</template>
```

## Props

All components accept the following props:

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `api-key` | string | Yes | - | Your GrooveShop API key |
| `tenant-id` | string | Yes | - | Your store identifier |
| `api-url` | string | No | - | API endpoint URL |
| `product-id` | string | No | - | Source product ID |
| `count` | number | No | 5 | Number of recommendations |
| `type` | RecommendationType | No | 'similar' | Type of recommendations |
| `theme` | 'light' \| 'dark' \| 'minimal' | No | 'light' | Visual theme |
| `real-time` | boolean | No | false | Enable real-time social proof |
| `user-id` | string | No | - | User ID for personalization |
| `test-id` | string | No | - | A/B test ID |
| `className` | string | No | '' | Additional CSS class |
| `style` | object | No | {} | Inline styles |

## Events

### @product-click

Emitted when a product is clicked.

```vue
<template>
  <RecommendationCarousel
    v-bind="props"
    @product-click="handleClick"
  />
</template>

<script setup>
function handleClick(product) {
  console.log('User clicked product:', product.entity_id);
  // Navigate, add to cart, etc.
}
</script>
```

### @product-impression

Emitted when products are displayed.

```vue
<template>
  <RecommendationCarousel
    v-bind="props"
    @product-impression="handleImpression"
  />
</template>

<script setup>
function handleImpression(products) {
  console.log('Widget displayed', products.length, 'products');
}
</script>
```

## Composable

### useRecommendations

Access the widget instance and listen to events.

```vue
<script setup>
import { useRecommendations } from '@grooveshop/recommendations-vue';
import { onMounted } from 'vue';

const { on, isInitialized } = useRecommendations({
  apiKey: 'pk_live_abc123',
  tenantId: 'my-store',
});

onMounted(() => {
  // Listen to all events
  const unsubscribe = on('*', (event) => {
    console.log('Event:', event.type, event);
  });

  // Cleanup on unmount
  return unsubscribe;
});
</script>
```

## Recommendation Types

- `similar` - Products similar to the current product
- `trending` - Trending products
- `bundle` - Frequently bought together
- `personalized` - Personalized for the user
- `complement` - Complementary products
- `recently-viewed` - Recently viewed by user
- `auto` - Auto-detect based on context

## Real-time Social Proof

```vue
<template>
  <RecommendationCarousel
    v-bind="props"
    :real-time="true"
  />
</template>
```

Shows live badges:
- "5 viewing now"
- "3 sold today"
- "2 in carts"

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

```vue
<template>
  <RecommendationCarousel
    v-bind="props"
    class-name="my-custom-class"
    :style="{ marginTop: '2rem' }"
  />
</template>
```

## TypeScript

Full TypeScript support with type definitions included.

```typescript
import type {
  Product,
  RecommendationType,
  RecommendationProps,
} from '@grooveshop/recommendations-vue';
```

## License

MIT
