=== GrooveShop Recommendations ===
Contributors: grooveshop
Tags: recommendations, personalization, woocommerce, products, ecommerce
Requires at least: 5.8
Tested up to: 6.4
Requires PHP: 7.4
Stable tag: 1.0.0
License: GPLv2 or later
License URI: https://www.gnu.org/licenses/gpl-2.0.html

AI-powered product recommendations for WooCommerce stores. Increase sales with personalized product suggestions.

== Description ==

GrooveShop Recommendations brings intelligent, AI-powered product recommendations to your WooCommerce store. Display similar products, trending items, bundles, and personalized suggestions to increase average order value and customer engagement.

### Key Features

* **Multiple Recommendation Types**
  * Similar Products - Show products related to the current item
  * Trending Products - Display what's popular right now
  * Bundles - Suggest complementary product combinations
  * Personalized - AI-powered recommendations based on user behavior
  * Complementary Products - Items that go well together
  * Recently Viewed - Help customers find products they've seen before

* **Flexible Display Options**
  * 3 Layout Styles: Carousel, Grid, List
  * 3 Theme Options: Light, Dark, Minimal
  * Customizable product count (1-20 items)
  * Responsive design for all devices

* **Easy Integration**
  * Gutenberg block for the block editor
  * Classic shortcode support
  * Automatic placement on product and cart pages
  * WooCommerce integration for order tracking

* **Real-time Features**
  * Live social proof ("5 people viewing this")
  * Real-time trending updates
  * Automatic click and impression tracking

* **Performance Optimized**
  * Lazy loading for fast page speeds
  * Image prefetching
  * Minimal JavaScript footprint (8.8 KB gzipped)
  * CDN-delivered widget

### Requirements

* WooCommerce 6.0 or higher
* GrooveShop account (get your free API key at [dashboard.grooveshop.com](https://dashboard.grooveshop.com))

### How It Works

1. Sign up for a free GrooveShop account
2. Install and activate the plugin
3. Enter your API credentials in Settings > GrooveShop
4. Add recommendation widgets using blocks or shortcodes
5. Watch your conversions grow!

### Developer Friendly

* Clean, documented code
* WordPress coding standards
* Action and filter hooks
* TypeScript definitions
* REST API integration

== Installation ==

### Automatic Installation

1. Log in to your WordPress admin panel
2. Go to Plugins > Add New
3. Search for "GrooveShop Recommendations"
4. Click "Install Now" and then "Activate"

### Manual Installation

1. Download the plugin ZIP file
2. Upload to `/wp-content/plugins/` directory
3. Activate the plugin through the 'Plugins' menu in WordPress
4. Go to Settings > GrooveShop to configure

### Configuration

1. Navigate to Settings > GrooveShop in your WordPress admin
2. Enter your API Key (starts with `pk_`)
3. Enter your Tenant ID (your store identifier)
4. Configure widget settings (layout, theme, count)
5. Enable auto-placement on product/cart pages if desired
6. Save settings

== Frequently Asked Questions ==

= Do I need a GrooveShop account? =

Yes, you need a GrooveShop account to use this plugin. Sign up for free at [dashboard.grooveshop.com](https://dashboard.grooveshop.com).

= Is there a free plan? =

Yes! GrooveShop offers a free tier with up to 10,000 monthly impressions, perfect for small stores getting started.

= How do I get my API key? =

After signing up for GrooveShop, you'll find your API key in your dashboard under Settings > API Keys.

= Does this work with other ecommerce platforms? =

This WordPress plugin is specifically for WooCommerce. GrooveShop also supports Shopify and custom integrations.

= Will this slow down my site? =

No! The widget is highly optimized at only 8.8 KB gzipped, uses lazy loading, and is served from a global CDN for maximum performance.

= Can I customize the appearance? =

Yes! You can choose from 3 layouts (carousel, grid, list) and 3 themes (light, dark, minimal). Advanced customization is available via CSS.

= Does it work with my theme? =

Yes! The widget is designed to work with any WordPress theme and automatically adapts to your site's styling.

= How accurate are the recommendations? =

Our AI-powered recommendation engine learns from your store's data and improves over time. Most stores see 15-30% increase in conversions.

= Can I track performance? =

Yes! Your GrooveShop dashboard provides detailed analytics on impressions, clicks, conversions, and revenue.

= What data is collected? =

We collect anonymized product views, clicks, and purchases to improve recommendations. No personally identifiable information is stored.

== Screenshots ==

1. Settings page with API configuration
2. Gutenberg block in the editor
3. Carousel layout on product page
4. Grid layout with light theme
5. List layout with dark theme
6. Real-time social proof badges
7. Dashboard analytics

== Changelog ==

= 1.0.0 - 2024-01-15 =
* Initial release
* Multiple recommendation types (similar, trending, bundle, personalized)
* 3 layout options (carousel, grid, list)
* 3 theme options (light, dark, minimal)
* Gutenberg block support
* Shortcode support
* WooCommerce integration
* Real-time social proof
* Auto-placement on product/cart pages
* Performance optimization (lazy loading, prefetching)
* Analytics integration

== Upgrade Notice ==

= 1.0.0 =
Initial release of GrooveShop Recommendations. Install now to start increasing your WooCommerce sales!

== Shortcode Usage ==

### Basic Usage

`[grooveshop_recommendations]`

### Similar Products

`[grooveshop_recommendations type="similar" product_id="123" count="5" layout="carousel"]`

### Trending Products

`[grooveshop_recommendations type="trending" count="8" layout="grid"]`

### Personalized Recommendations

`[grooveshop_recommendations type="personalized" count="6"]`

### Bundle Suggestions

`[grooveshop_recommendations type="bundle" count="4" layout="list"]`

### All Parameters

* `type` - Recommendation type: similar, trending, bundle, personalized, complement, recently-viewed, auto (default: similar)
* `product_id` - Product ID for similar/complement types (auto-detected on product pages)
* `count` - Number of products to show, 1-20 (default: 5)
* `layout` - Display layout: carousel, grid, list (default: carousel)
* `theme` - Visual theme: light, dark, minimal (default: light)
* `real_time` - Enable real-time updates: true, false (default: false)

== Support ==

Need help? Check out our resources:

* [Documentation](https://docs.grooveshop.com)
* [Support Portal](https://support.grooveshop.com)
* [API Reference](https://api.grooveshop.com/docs)
* [GitHub Repository](https://github.com/grooveshop/recommendations)

== Privacy Policy ==

GrooveShop Recommendations collects the following data to provide personalized recommendations:

* Product views (anonymized)
* Click events on recommendations
* Purchase data (product IDs, quantities, prices)
* User session identifiers (anonymized)

We do NOT collect:

* Personal information (names, emails, addresses)
* Payment information
* Browsing history outside your store

All data is encrypted in transit and at rest. See our full privacy policy at [grooveshop.com/privacy](https://grooveshop.com/privacy).
