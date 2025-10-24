# 🚀 GrooveShop Recommendations Widget

Plug-and-play AI-powered product recommendations for any website. No framework required!

## Features

- ✅ **Zero dependencies** - Pure TypeScript/JavaScript
- ✅ **Universal** - Works with WordPress, Shopify, plain HTML, React, Vue, etc.
- ✅ **3 layouts** - Carousel, Grid, List
- ✅ **Smart auto-detection** - Automatically finds product IDs
- ✅ **Event tracking** - Built-in analytics
- ✅ **Responsive** - Mobile-first design
- ✅ **Themeable** - CSS custom properties
- ✅ **TypeScript** - Full type safety

## Quick Start

### 1. Add Script Tag

```html
<script src="https://cdn.grooveshop.com/widget/v1/widget.umd.js"
        data-api-key="YOUR_API_KEY"
        data-tenant-id="your-store"></script>
```

### 2. Add Widget

```html
<!-- Similar products carousel -->
<div data-grooveshop-recommendations
     data-product-id="123"
     data-count="5"
     data-layout="carousel"></div>
```

That's it! ✨

## Layouts

### Carousel
```html
<div data-grooveshop-recommendations
     data-product-id="123"
     data-layout="carousel"
     data-count="8"></div>
```

### Grid
```html
<div data-grooveshop-recommendations
     data-product-id="123"
     data-layout="grid"
     data-count="6"></div>
```

### List
```html
<div data-grooveshop-recommendations
     data-product-id="123"
     data-layout="list"
     data-count="4"></div>
```

## Recommendation Types

### Similar Products (default)
```html
<div data-grooveshop-recommendations
     data-product-id="123"
     data-type="similar"></div>
```

### Trending Products
```html
<div data-grooveshop-recommendations
     data-type="trending"
     data-count="10"></div>
```

### Auto Mode (Smart)
```html
<!-- Automatically detects product ID and context -->
<div data-grooveshop-recommendations
     data-type="auto"></div>
```

## Configuration

### Script Tag Options

| Attribute | Required | Default | Description |
|-----------|----------|---------|-------------|
| `data-api-key` | ✅ Yes | - | Your API key |
| `data-tenant-id` | ✅ Yes | - | Your store ID |
| `data-api-url` | ❌ No | `https://api.grooveshop.com` | API endpoint |
| `data-auto-track` | ❌ No | `true` | Auto-track events |
| `data-debug` | ❌ No | `false` | Console logging |

### Widget Attributes

| Attribute | Required | Default | Description |
|-----------|----------|---------|-------------|
| `data-product-id` | ⚠️ Conditional | - | Source product ID (required for `similar`) |
| `data-count` | ❌ No | `5` | Number of products |
| `data-layout` | ❌ No | `grid` | Layout: `carousel`, `grid`, or `list` |
| `data-type` | ❌ No | `similar` | Type: `similar`, `trending`, or `auto` |
| `data-theme` | ❌ No | `light` | Theme: `light`, `dark`, or `minimal` |

## Customization

### CSS Custom Properties

```css
:root {
  --gs-primary-color: #007bff;
  --gs-primary-hover: #0056b3;
  --gs-border-radius: 8px;
  --gs-font-family: 'Your Font', sans-serif;
}
```

### Override Styles

```css
.gs-card {
  border: 2px solid #007bff;
}

.gs-card-title {
  font-size: 18px;
  color: #333;
}
```

## Development

### Prerequisites

- Node.js 18+
- npm or yarn

### Setup

```bash
cd widget
npm install
```

### Development Server

```bash
npm run dev
```

Opens at `http://localhost:3001` with hot reload.

### Build

```bash
npm run build
```

Creates:
- `dist/widget.umd.js` - UMD bundle for browsers
- `dist/widget.es.js` - ES module
- `dist/widget.css` - Styles
- `dist/index.d.ts` - TypeScript definitions

### Preview Build

```bash
npm run preview
```

## Architecture

```
widget/
├── src/
│   ├── index.ts              # Main entry point
│   ├── config.ts             # Configuration loader
│   ├── api.ts                # API client
│   ├── types.ts              # TypeScript types
│   ├── widgets/
│   │   ├── carousel.ts       # Carousel renderer
│   │   ├── grid.ts           # Grid renderer
│   │   └── list.ts           # List renderer
│   ├── templates/
│   │   └── card.ts           # Product card template
│   └── styles/
│       └── base.css          # Widget styles
├── dist/                     # Build output
├── index.html                # Demo page
├── vite.config.ts            # Vite configuration
├── tsconfig.json             # TypeScript config
└── package.json
```

## Browser Support

- Chrome (last 2 versions)
- Firefox (last 2 versions)
- Safari (last 2 versions)
- Edge (last 2 versions)

## License

MIT

## Support

- 📧 Email: support@grooveshop.com
- 💬 Discord: [Join our community](https://discord.gg/grooveshop)
- 📚 Docs: [docs.grooveshop.com](https://docs.grooveshop.com)
