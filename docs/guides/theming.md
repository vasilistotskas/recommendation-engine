# Theming Guide

Customize the appearance of GrooveShop recommendation widgets to match your brand.

## Table of Contents

- [Quick Theming](#quick-theming)
- [CSS Custom Properties](#css-custom-properties)
- [Built-in Themes](#built-in-themes)
- [Custom Themes](#custom-themes)
- [Component Styling](#component-styling)
- [Responsive Design](#responsive-design)
- [Dark Mode](#dark-mode)
- [Examples](#examples)

---

## Quick Theming

### Method 1: CSS Custom Properties (Recommended)

The easiest way to customize colors, fonts, and spacing:

```html
<style>
  .gs-recommendations {
    --gs-primary-color: #e74c3c;
    --gs-font-family: 'Poppins', sans-serif;
    --gs-border-radius: 12px;
  }
</style>
```

### Method 2: Override CSS Classes

For more control, override specific CSS classes:

```html
<style>
  .gs-card {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .gs-card-title {
    font-size: 18px;
    font-weight: 600;
  }
</style>
```

### Method 3: Use Built-in Themes

Choose from 3 built-in themes:

```html
<!-- Light theme (default) -->
<div data-grooveshop-recommendations data-theme="light"></div>

<!-- Dark theme -->
<div data-grooveshop-recommendations data-theme="dark"></div>

<!-- Minimal theme -->
<div data-grooveshop-recommendations data-theme="minimal"></div>
```

---

## CSS Custom Properties

All available CSS variables with their default values:

### Colors

```css
.gs-recommendations {
  /* Primary color (buttons, links) */
  --gs-primary-color: #007bff;
  --gs-primary-hover: #0056b3;

  /* Text colors */
  --gs-text-color: #333333;
  --gs-text-secondary: #666666;
  --gs-text-muted: #999999;

  /* Background colors */
  --gs-bg-color: #ffffff;
  --gs-bg-secondary: #f8f9fa;
  --gs-bg-hover: #f1f3f5;

  /* Border colors */
  --gs-border-color: #dee2e6;
  --gs-border-hover: #adb5bd;

  /* Sale/discount colors */
  --gs-sale-color: #e74c3c;
  --gs-sale-bg: #ffe5e5;

  /* Success/stock colors */
  --gs-success-color: #28a745;
  --gs-warning-color: #ffc107;

  /* Social proof colors */
  --gs-social-proof-bg: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  --gs-social-proof-text: #ffffff;
}
```

### Typography

```css
.gs-recommendations {
  /* Font families */
  --gs-font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --gs-font-family-heading: inherit;

  /* Font sizes */
  --gs-font-size-base: 14px;
  --gs-font-size-small: 12px;
  --gs-font-size-large: 16px;
  --gs-font-size-heading: 18px;

  /* Font weights */
  --gs-font-weight-normal: 400;
  --gs-font-weight-medium: 500;
  --gs-font-weight-bold: 600;

  /* Line heights */
  --gs-line-height-base: 1.5;
  --gs-line-height-heading: 1.2;
}
```

### Spacing

```css
.gs-recommendations {
  /* Padding */
  --gs-spacing-xs: 4px;
  --gs-spacing-sm: 8px;
  --gs-spacing-md: 16px;
  --gs-spacing-lg: 24px;
  --gs-spacing-xl: 32px;

  /* Card spacing */
  --gs-card-padding: 16px;
  --gs-card-gap: 16px;
}
```

### Layout

```css
.gs-recommendations {
  /* Border radius */
  --gs-border-radius: 8px;
  --gs-border-radius-sm: 4px;
  --gs-border-radius-lg: 12px;

  /* Shadows */
  --gs-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.1);
  --gs-shadow-md: 0 2px 8px rgba(0, 0, 0, 0.1);
  --gs-shadow-lg: 0 4px 16px rgba(0, 0, 0, 0.15);

  /* Card dimensions */
  --gs-card-width: 220px;
  --gs-card-image-height: 220px;

  /* Transitions */
  --gs-transition-speed: 0.2s;
  --gs-transition-easing: ease-in-out;
}
```

---

## Built-in Themes

### Light Theme (Default)

Clean, bright design suitable for most sites.

```html
<div data-grooveshop-recommendations data-theme="light"></div>
```

**Characteristics:**
- White background
- Dark text
- Subtle shadows
- Colorful accents

---

### Dark Theme

Modern dark mode design.

```html
<div data-grooveshop-recommendations data-theme="dark"></div>
```

**Characteristics:**
- Dark background (#1a1a1a)
- Light text
- Reduced contrast shadows
- Vibrant accents

---

### Minimal Theme

Borderless, shadow-free design.

```html
<div data-grooveshop-recommendations data-theme="minimal"></div>
```

**Characteristics:**
- No borders
- No shadows
- Clean typography
- Subtle hover states

---

## Custom Themes

### Creating a Custom Theme

```css
.gs-recommendations {
  /* Brand colors */
  --gs-primary-color: #ff6b6b;
  --gs-primary-hover: #ee5555;

  /* Custom font */
  --gs-font-family: 'Inter', -apple-system, sans-serif;
  --gs-font-weight-bold: 700;

  /* Rounded design */
  --gs-border-radius: 16px;

  /* Playful shadows */
  --gs-shadow-md: 0 8px 24px rgba(255, 107, 107, 0.2);

  /* Tight spacing */
  --gs-card-padding: 12px;
  --gs-card-gap: 12px;
}
```

### Theme Presets

#### E-commerce Modern

```css
.gs-recommendations {
  --gs-primary-color: #0070f3;
  --gs-border-radius: 8px;
  --gs-font-family: 'Inter', sans-serif;
  --gs-shadow-md: 0 4px 12px rgba(0, 0, 0, 0.08);
}
```

#### Fashion Boutique

```css
.gs-recommendations {
  --gs-primary-color: #d4af37;
  --gs-border-radius: 0;
  --gs-font-family: 'Playfair Display', serif;
  --gs-font-weight-bold: 700;
  --gs-text-color: #2c2c2c;
}
```

#### Tech Minimal

```css
.gs-recommendations {
  --gs-primary-color: #000000;
  --gs-border-radius: 4px;
  --gs-font-family: 'SF Pro Display', -apple-system, sans-serif;
  --gs-shadow-md: none;
  --gs-border-color: #e0e0e0;
}
```

#### Vibrant Youth

```css
.gs-recommendations {
  --gs-primary-color: #ff4081;
  --gs-border-radius: 20px;
  --gs-font-family: 'Poppins', sans-serif;
  --gs-shadow-md: 0 8px 32px rgba(255, 64, 129, 0.3);
  --gs-card-padding: 20px;
}
```

---

## Component Styling

### Product Cards

```css
.gs-card {
  /* Override card styles */
  background: #ffffff;
  border: 1px solid var(--gs-border-color);
  border-radius: var(--gs-border-radius);
  padding: var(--gs-card-padding);
  transition: all var(--gs-transition-speed);
}

.gs-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--gs-shadow-lg);
}
```

### Product Images

```css
.gs-card-image {
  border-radius: var(--gs-border-radius-sm);
  overflow: hidden;
}

.gs-card-image img {
  transition: transform 0.3s;
}

.gs-card:hover .gs-card-image img {
  transform: scale(1.05);
}
```

### Product Titles

```css
.gs-card-title {
  font-size: var(--gs-font-size-heading);
  font-weight: var(--gs-font-weight-bold);
  color: var(--gs-text-color);
  margin: 12px 0 8px 0;
  line-height: var(--gs-line-height-heading);
}
```

### Prices

```css
.gs-card-price {
  font-size: var(--gs-font-size-large);
  font-weight: var(--gs-font-weight-bold);
  color: var(--gs-primary-color);
}

.gs-price-old {
  text-decoration: line-through;
  color: var(--gs-text-muted);
  font-size: var(--gs-font-size-base);
  margin-right: 8px;
}

.gs-price-sale {
  color: var(--gs-sale-color);
}
```

### Buttons

```css
.gs-btn {
  background: var(--gs-primary-color);
  color: #ffffff;
  border: none;
  border-radius: var(--gs-border-radius-sm);
  padding: 10px 20px;
  font-weight: var(--gs-font-weight-medium);
  cursor: pointer;
  transition: background var(--gs-transition-speed);
}

.gs-btn:hover {
  background: var(--gs-primary-hover);
}
```

### Carousel Navigation

```css
.gs-carousel-arrow {
  background: var(--gs-bg-color);
  border: 1px solid var(--gs-border-color);
  border-radius: 50%;
  width: 40px;
  height: 40px;
  cursor: pointer;
  transition: all var(--gs-transition-speed);
}

.gs-carousel-arrow:hover {
  background: var(--gs-primary-color);
  border-color: var(--gs-primary-color);
  color: #ffffff;
}
```

---

## Responsive Design

### Mobile-First Approach

```css
/* Mobile (default) */
.gs-recommendations {
  --gs-card-width: 160px;
  --gs-card-padding: 12px;
  --gs-font-size-heading: 14px;
}

/* Tablet */
@media (min-width: 768px) {
  .gs-recommendations {
    --gs-card-width: 200px;
    --gs-card-padding: 16px;
    --gs-font-size-heading: 16px;
  }
}

/* Desktop */
@media (min-width: 1024px) {
  .gs-recommendations {
    --gs-card-width: 220px;
    --gs-card-padding: 20px;
    --gs-font-size-heading: 18px;
  }
}
```

### Grid Breakpoints

```css
/* Customize grid columns */
.gs-grid {
  display: grid;
  gap: var(--gs-card-gap);
  grid-template-columns: repeat(2, 1fr);  /* Mobile: 2 columns */
}

@media (min-width: 768px) {
  .gs-grid {
    grid-template-columns: repeat(3, 1fr);  /* Tablet: 3 columns */
  }
}

@media (min-width: 1024px) {
  .gs-grid {
    grid-template-columns: repeat(4, 1fr);  /* Desktop: 4 columns */
  }
}
```

---

## Dark Mode

### Automatic Dark Mode

Automatically switch based on user's system preference:

```css
@media (prefers-color-scheme: dark) {
  .gs-recommendations {
    --gs-bg-color: #1a1a1a;
    --gs-bg-secondary: #2a2a2a;
    --gs-text-color: #ffffff;
    --gs-text-secondary: #cccccc;
    --gs-border-color: #404040;
  }
}
```

### Manual Dark Mode Toggle

```css
/* Add class to enable dark mode */
.dark-mode .gs-recommendations {
  --gs-bg-color: #1a1a1a;
  --gs-text-color: #ffffff;
  --gs-border-color: #404040;
}
```

```javascript
// Toggle dark mode
document.getElementById('dark-mode-toggle').addEventListener('click', () => {
  document.body.classList.toggle('dark-mode');
});
```

---

## Examples

### Example 1: Brand Color Override

```html
<style>
  .gs-recommendations {
    --gs-primary-color: #ff6b6b;
    --gs-primary-hover: #ee5555;
  }
</style>

<div data-grooveshop-recommendations
     data-type="trending"
     data-count="6">
</div>
```

### Example 2: Custom Font

```html
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" rel="stylesheet">

<style>
  .gs-recommendations {
    --gs-font-family: 'Inter', sans-serif;
    --gs-font-weight-bold: 600;
  }
</style>
```

### Example 3: Rounded Cards

```html
<style>
  .gs-recommendations {
    --gs-border-radius: 20px;
    --gs-border-radius-sm: 16px;
  }

  .gs-card {
    overflow: hidden;
  }
</style>
```

### Example 4: No Shadows (Flat Design)

```html
<style>
  .gs-recommendations {
    --gs-shadow-sm: none;
    --gs-shadow-md: none;
    --gs-shadow-lg: none;
    --gs-border-color: #e0e0e0;
  }

  .gs-card {
    border: 1px solid var(--gs-border-color);
  }
</style>
```

### Example 5: Compact Mobile Design

```html
<style>
  @media (max-width: 767px) {
    .gs-recommendations {
      --gs-card-width: 140px;
      --gs-card-padding: 8px;
      --gs-font-size-heading: 13px;
      --gs-font-size-base: 12px;
      --gs-card-gap: 8px;
    }
  }
</style>
```

### Example 6: Luxury/Premium Theme

```html
<link href="https://fonts.googleapis.com/css2?family=Playfair+Display:wght@400;600;700&display=swap" rel="stylesheet">

<style>
  .gs-recommendations {
    --gs-font-family-heading: 'Playfair Display', serif;
    --gs-primary-color: #d4af37;
    --gs-border-radius: 0;
    --gs-text-color: #2c2c2c;
    --gs-shadow-md: 0 2px 16px rgba(212, 175, 55, 0.1);
  }

  .gs-card-title {
    font-family: var(--gs-font-family-heading);
    text-transform: uppercase;
    letter-spacing: 1px;
  }
</style>
```

---

## CSS Class Reference

All CSS classes used by the widget:

### Container Classes
- `.gs-recommendations` - Main container
- `.gs-carousel` - Carousel layout container
- `.gs-grid` - Grid layout container
- `.gs-list` - List layout container

### Card Classes
- `.gs-card` - Product card
- `.gs-card-image` - Image container
- `.gs-card-content` - Content container
- `.gs-card-title` - Product title
- `.gs-card-price` - Price container
- `.gs-card-rating` - Rating stars
- `.gs-card-badge` - Discount/sale badge

### Price Classes
- `.gs-price` - Regular price
- `.gs-price-old` - Strikethrough old price
- `.gs-price-sale` - Sale price

### Button Classes
- `.gs-btn` - Primary button
- `.gs-btn-secondary` - Secondary button

### Navigation Classes
- `.gs-carousel-arrow` - Arrow buttons
- `.gs-carousel-arrow-prev` - Previous button
- `.gs-carousel-arrow-next` - Next button

### Badge Classes
- `.gs-badge` - Generic badge
- `.gs-badge-sale` - Sale badge
- `.gs-badge-new` - New product badge
- `.gs-social-proof` - Social proof badge

### State Classes
- `.gs-loading` - Loading state
- `.gs-error` - Error state
- `.gs-empty` - No products state

---

## Tips & Best Practices

### 1. Use CSS Variables

Always prefer CSS variables over hardcoded values for easy maintenance:

```css
/* Good */
.gs-card {
  padding: var(--gs-card-padding);
}

/* Bad */
.gs-card {
  padding: 16px;
}
```

### 2. Maintain Contrast

Ensure sufficient color contrast for accessibility (WCAG AA minimum 4.5:1):

```css
/* Good contrast */
--gs-text-color: #333333;
--gs-bg-color: #ffffff;

/* Poor contrast (avoid) */
--gs-text-color: #cccccc;
--gs-bg-color: #ffffff;
```

### 3. Test on Mobile

Always test your custom theme on mobile devices:

```css
/* Mobile-friendly card width */
@media (max-width: 767px) {
  .gs-recommendations {
    --gs-card-width: min(160px, 45vw);
  }
}
```

### 4. Use System Fonts for Performance

System fonts load instantly:

```css
--gs-font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto,
                  'Helvetica Neue', Arial, sans-serif;
```

### 5. Scope Your Overrides

Scope custom styles to avoid conflicts:

```css
/* Scoped to specific page */
.product-page .gs-recommendations {
  --gs-primary-color: #custom-color;
}
```

---

## Troubleshooting

### Styles Not Applying?

1. Check CSS specificity
2. Ensure styles load after widget
3. Use `!important` if needed (sparingly)
4. Clear browser cache

### Colors Not Changing?

```css
/* Make sure to use CSS variables correctly */
.gs-recommendations {
  --gs-primary-color: #ff0000 !important;
}
```

### Mobile Layout Broken?

```css
/* Ensure responsive variables are set */
@media (max-width: 767px) {
  .gs-recommendations {
    --gs-card-width: 160px;
  }
}
```

---

## See Also

- [Configuration Guide](./configuration.md)
- [Layout Options](./layouts.md)
- [Custom Templates](./custom-templates.md)
- [Widget API Reference](../api/widget-api.md)
