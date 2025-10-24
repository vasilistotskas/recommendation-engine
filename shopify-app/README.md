# GrooveShop Recommendations - Shopify App

AI-powered product recommendations for Shopify stores. Seamlessly integrates with your Shopify admin and theme to display personalized product suggestions.

## Features

- **Easy Setup**: Install from Shopify App Store and configure in minutes
- **Theme App Extension**: Drag-and-drop recommendations blocks in your theme editor
- **Auto-sync**: Automatically syncs products and orders to GrooveShop
- **Multiple Recommendation Types**: Similar, trending, bundles, personalized, and more
- **Flexible Display**: Choose from carousel, grid, or list layouts
- **Real-time Updates**: Live social proof and trending updates
- **Analytics Dashboard**: Track performance in your GrooveShop dashboard

## Installation

### From Shopify App Store

1. Visit the [GrooveShop Recommendations app listing](https://apps.shopify.com/grooveshop-recommendations)
2. Click "Add app"
3. Follow the installation prompts
4. Configure your settings in the app

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/grooveshop/recommendations.git
   cd recommendations/shopify-app
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Set up environment variables:
   ```bash
   cp .env.example .env
   # Edit .env with your credentials
   ```

4. Set up database:
   ```bash
   npx prisma migrate dev
   ```

5. Start development server:
   ```bash
   npm run dev
   ```

## Configuration

### 1. Get API Credentials

1. Sign up at [dashboard.grooveshop.com](https://dashboard.grooveshop.com)
2. Create a new store
3. Copy your API Key and Tenant ID

### 2. Configure App

1. Open the app in your Shopify admin
2. Navigate to Settings
3. Enter your API Key and Tenant ID
4. Configure widget preferences:
   - Layout (carousel, grid, list)
   - Number of products to show
   - Theme (light, dark, minimal)
   - Auto-placement options

### 3. Add to Theme

#### Using Theme App Extension (Recommended)

1. Go to your theme editor
2. Navigate to the page where you want recommendations
3. Click "Add block" or "Add section"
4. Choose "GrooveShop Recommendations"
5. Configure block settings:
   - Recommendation type
   - Layout
   - Number of products
   - Section heading

#### Using Liquid Code

Add this to your theme where you want recommendations:

```liquid
{% render 'grooveshop-recommendations',
  type: 'similar',
  layout: 'carousel',
  count: 5,
  theme: 'light'
%}
```

## Recommendation Types

### Similar Products
```liquid
{% render 'grooveshop-recommendations',
  type: 'similar',
  product_id: product.id
%}
```
Shows products similar to the specified product. Auto-detects product ID on product pages.

### Trending Products
```liquid
{% render 'grooveshop-recommendations',
  type: 'trending',
  count: 8
%}
```
Displays currently trending products across your store.

### Personalized
```liquid
{% render 'grooveshop-recommendations',
  type: 'personalized',
  user_id: customer.id
%}
```
AI-powered recommendations based on customer behavior.

### Bundle
```liquid
{% render 'grooveshop-recommendations',
  type: 'bundle'
%}
```
Suggests product bundles for the current cart contents.

### Complementary
```liquid
{% render 'grooveshop-recommendations',
  type: 'complement',
  product_id: product.id
%}
```
Shows products that complement the specified product.

### Recently Viewed
```liquid
{% render 'grooveshop-recommendations',
  type: 'recently-viewed',
  count: 6
%}
```
Displays products the customer recently viewed.

## Auto-Placement

Enable auto-placement in app settings to automatically show:

- **Product Pages**: Similar products below product description
- **Cart Page**: Bundle suggestions in cart

## Webhooks

The app automatically registers webhooks for:

- `APP_UNINSTALLED` - Cleanup when app is uninstalled
- `PRODUCTS_CREATE` - Sync new products
- `PRODUCTS_UPDATE` - Update product data
- `PRODUCTS_DELETE` - Remove deleted products
- `ORDERS_CREATE` - Track purchases

## API Endpoints

### Settings
- `GET /` - Dashboard home
- `GET /settings` - Settings page
- `POST /settings` - Save settings

### Webhooks
- `POST /webhooks` - Webhook handler

## Development

### Tech Stack

- **Framework**: Remix
- **UI**: Shopify Polaris
- **Database**: PostgreSQL + Prisma
- **Authentication**: Shopify OAuth
- **Deployment**: Fly.io

### Project Structure

```
shopify-app/
├── app/
│   ├── routes/
│   │   ├── _index.tsx        # Dashboard
│   │   ├── settings.tsx      # Settings page
│   │   └── webhooks.tsx      # Webhook handler
│   └── shopify.server.ts     # Shopify config
├── extensions/
│   └── recommendations-widget/
│       ├── blocks/
│       │   └── recommendations.liquid
│       ├── assets/
│       │   └── widget-loader.js
│       └── shopify.extension.toml
├── prisma/
│   └── schema.prisma         # Database schema
├── server/
│   ├── index.ts             # Shopify app setup
│   └── db.server.ts         # Database client
└── shopify.app.toml         # App configuration
```

### Database Schema

- **Session** - Shopify session storage
- **Store** - Store settings and API credentials
- **Product** - Product sync tracking
- **Order** - Order tracking

### Testing

```bash
# Type checking
npm run typecheck

# Build
npm run build

# Deploy
npm run deploy
```

## Deployment

### Using Fly.io

1. Install Fly CLI:
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. Login:
   ```bash
   fly auth login
   ```

3. Create app:
   ```bash
   fly launch
   ```

4. Set secrets:
   ```bash
   fly secrets set SHOPIFY_API_KEY=your_key
   fly secrets set SHOPIFY_API_SECRET=your_secret
   fly secrets set DATABASE_URL=your_db_url
   ```

5. Deploy:
   ```bash
   fly deploy
   ```

### Environment Variables

Required:
- `SHOPIFY_API_KEY` - Your Shopify API key
- `SHOPIFY_API_SECRET` - Your Shopify API secret
- `SHOPIFY_APP_URL` - Your app URL
- `DATABASE_URL` - PostgreSQL connection string
- `SESSION_SECRET` - Random secret for sessions

Optional:
- `SCOPES` - Comma-separated OAuth scopes (default: read_products,write_products,read_orders,write_script_tags,read_customers,write_themes)
- `GROOVESHOP_API_URL` - GrooveShop API URL (default: https://api.grooveshop.com)

## Support

- **Documentation**: [docs.grooveshop.com/shopify](https://docs.grooveshop.com/shopify)
- **Support**: [support.grooveshop.com](https://support.grooveshop.com)
- **Dashboard**: [dashboard.grooveshop.com](https://dashboard.grooveshop.com)
- **GitHub**: [github.com/grooveshop/recommendations](https://github.com/grooveshop/recommendations)

## License

MIT License - see LICENSE file for details
