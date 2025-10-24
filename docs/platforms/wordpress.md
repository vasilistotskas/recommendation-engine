# WordPress Integration Guide

Complete guide to using GrooveShop Recommendations with WordPress and WooCommerce.

## Installation

### From WordPress.org (Recommended)

1. Log in to WordPress admin
2. Go to **Plugins → Add New**
3. Search for "GrooveShop Recommendations"
4. Click **Install Now**
5. Click **Activate**

### Manual Installation

1. Download the plugin ZIP from [WordPress.org](https://wordpress.org/plugins/grooveshop-recommendations)
2. Go to **Plugins → Add New → Upload Plugin**
3. Choose the ZIP file and click **Install Now**
4. Click **Activate Plugin**

---

## Configuration

### 1. Get API Credentials

1. Sign up at [dashboard.grooveshop.com](https://dashboard.grooveshop.com)
2. Create a new store
3. Copy your **API Key** (starts with `pk_`)
4. Copy your **Tenant ID**

### 2. Configure Plugin Settings

1. Go to **Settings → GrooveShop** in WordPress admin
2. Enter your **API Key**
3. Enter your **Tenant ID**
4. Click **Save Settings**

The plugin will automatically validate your credentials!

### 3. Configure Widget Settings

In the same settings page:

- **Layout**: Choose carousel, grid, or list
- **Products to Show**: 1-20 (default: 5)
- **Theme**: Light, dark, or minimal
- **Auto-placement**: Enable for automatic widget display
- **Tracking**: Enable automatic click and impression tracking

---

## Usage

### Method 1: Gutenberg Blocks (Easiest)

1. Edit any page or post
2. Click **+** to add a block
3. Search for "GrooveShop Recommendations"
4. Add the block
5. Configure settings in the sidebar:
   - Recommendation Type
   - Number of Products
   - Layout
   - Theme

6. **Publish** or **Update**

**Perfect for:** Non-technical users, visual editors

---

### Method 2: Shortcodes

Add shortcodes anywhere in your content:

#### Basic Usage

```php
[grooveshop_recommendations]
```

#### Similar Products

```php
[grooveshop_recommendations type="similar" product_id="123" count="5"]
```

#### Trending Products

```php
[grooveshop_recommendations type="trending" count="8" layout="grid"]
```

#### Personalized Recommendations

```php
[grooveshop_recommendations type="personalized" count="6"]
```

#### All Parameters

```php
[grooveshop_recommendations
  type="similar"
  product_id="123"
  count="5"
  layout="carousel"
  theme="light"
  real_time="false"]
```

**Perfect for:** Classic editor users, template customization

---

### Method 3: Auto-Placement

Enable in **Settings → GrooveShop**:

- **Show similar products on product pages**: Automatically displays below product description
- **Show bundles on cart page**: Automatically displays in cart

**Perfect for:** Quick setup, hands-off approach

---

### Method 4: PHP Templates

Add to your theme files:

```php
<?php
// In single-product.php
echo do_shortcode('[grooveshop_recommendations type="similar" count="5"]');
?>
```

```php
<?php
// In cart.php
echo do_shortcode('[grooveshop_recommendations type="bundle" count="4"]');
?>
```

```php
<?php
// In front-page.php
echo do_shortcode('[grooveshop_recommendations type="trending" count="8" layout="grid"]');
?>
```

**Perfect for:** Theme developers, custom templates

---

## WooCommerce Integration

The plugin automatically integrates with WooCommerce:

### Automatic Product Sync

Products are automatically synced to GrooveShop when:
- Product is created
- Product is updated
- Product details change

**No manual work required!**

### Automatic Order Tracking

Orders are automatically tracked when:
- Order is completed
- Payment is received

**Tracked data:**
- Products purchased
- Quantities
- Prices
- User ID (if logged in)

### Product Page Integration

Add recommendations to product pages:

**Option 1:** Enable auto-placement in settings

**Option 2:** Use shortcode in product description:
```php
[grooveshop_recommendations type="similar"]
```

**Option 3:** Edit `single-product.php`:
```php
<?php
// After product summary
do_action('woocommerce_after_single_product_summary');

// Add recommendations
echo do_shortcode('[grooveshop_recommendations type="similar" count="5"]');
?>
```

### Cart Page Integration

Add bundle suggestions to cart:

**Option 1:** Enable auto-placement in settings

**Option 2:** Edit `cart.php`:
```php
<div class="cart-collaterals">
  <h2>Frequently Bought Together</h2>
  <?php echo do_shortcode('[grooveshop_recommendations type="bundle" count="4"]'); ?>
</div>
```

### Checkout Page

Add last-chance upsells:

```php
// In checkout template
<?php
echo do_shortcode('[grooveshop_recommendations type="complement" count="3" layout="list"]');
?>
```

---

## Shortcode Reference

### Parameters

| Parameter | Type | Default | Options |
|-----------|------|---------|---------|
| `type` | string | `similar` | `similar`, `trending`, `bundle`, `personalized`, `complement`, `recently-viewed`, `auto` |
| `product_id` | string | auto | Any product ID |
| `count` | integer | `5` | `1-20` |
| `layout` | string | `carousel` | `carousel`, `grid`, `list` |
| `theme` | string | `light` | `light`, `dark`, `minimal` |
| `real_time` | string | `false` | `true`, `false` |

### Examples by Use Case

#### Product Pages

```php
<!-- Similar products -->
[grooveshop_recommendations type="similar" count="5"]

<!-- Complementary products -->
[grooveshop_recommendations type="complement" count="4"]
```

#### Homepage

```php
<!-- Trending products -->
[grooveshop_recommendations type="trending" count="8" layout="grid"]

<!-- Personalized (if user logged in) -->
[grooveshop_recommendations type="personalized" count="12" layout="grid"]
```

#### Cart Page

```php
<!-- Bundle suggestions -->
[grooveshop_recommendations type="bundle" count="4"]
```

#### Category Pages

```php
<!-- Trending in category -->
[grooveshop_recommendations type="trending" count="6" layout="grid"]
```

---

## Customization

### Custom Styling

Add to your theme's `style.css` or **Appearance → Customize → Additional CSS**:

```css
/* Change primary color */
.gs-recommendations {
  --gs-primary-color: #e74c3c;
  --gs-border-radius: 12px;
}

/* Customize card hover effect */
.gs-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

/* Change font */
.gs-recommendations {
  --gs-font-family: 'Your Theme Font', sans-serif;
}
```

### Widget Placement

Use WordPress hooks:

```php
// Add to functions.php

// Add after product content
add_action('woocommerce_after_single_product_summary', 'add_similar_products', 15);
function add_similar_products() {
  echo '<h2>You May Also Like</h2>';
  echo do_shortcode('[grooveshop_recommendations type="similar" count="5"]');
}

// Add to cart page
add_action('woocommerce_after_cart_table', 'add_cart_upsell');
function add_cart_upsell() {
  echo '<h3>Complete Your Purchase</h3>';
  echo do_shortcode('[grooveshop_recommendations type="bundle" count="4"]');
}
```

### Conditional Display

```php
<?php
// Show only on specific products
if (is_product() && has_term('electronics', 'product_cat')) {
  echo do_shortcode('[grooveshop_recommendations type="similar" count="5"]');
}
?>
```

```php
<?php
// Show only for logged-in users
if (is_user_logged_in()) {
  echo do_shortcode('[grooveshop_recommendations type="personalized" count="6"]');
}
?>
```

---

## Theme Integration

### Storefront Theme

```php
// Add to functions.php
add_action('storefront_after_content', 'add_recommendations');
function add_recommendations() {
  if (is_product()) {
    echo '<section class="recommendations">';
    echo '<h2>Recommended For You</h2>';
    echo do_shortcode('[grooveshop_recommendations type="similar" count="5"]');
    echo '</section>';
  }
}
```

### Astra Theme

```php
// Add to Astra child theme functions.php
add_action('astra_entry_content_after', 'add_recommendations');
function add_recommendations() {
  if (is_product()) {
    echo do_shortcode('[grooveshop_recommendations type="similar"]');
  }
}
```

### Divi Theme

Use the **Code Module**:
1. Add a **Code** module to your page
2. Paste the shortcode
3. Save

Or add to `functions.php`:
```php
add_action('wp_footer', 'add_recommendations');
function add_recommendations() {
  if (is_product()) {
    echo do_shortcode('[grooveshop_recommendations type="similar"]');
  }
}
```

---

## Troubleshooting

### Widgets Not Showing?

1. **Check API credentials**: Go to Settings → GrooveShop
2. **Clear cache**: If using caching plugin, clear all caches
3. **Check shortcode**: Ensure shortcode syntax is correct
4. **View source**: Look for `data-grooveshop-recommendations` in HTML
5. **Browser console**: Check for JavaScript errors (F12)

### API Validation Failed?

- Verify API key starts with `pk_`
- Ensure no extra spaces in credentials
- Check key is active in dashboard
- Try saving settings again

### Products Not Syncing?

- Ensure WooCommerce is active
- Check product is published (not draft)
- View network tab in browser console
- Contact support with product ID

### Styling Issues?

- Check for theme CSS conflicts
- Use `!important` in custom CSS if needed
- Disable other optimization plugins temporarily
- Test with a default theme (Twenty Twenty-Four)

---

## Performance Tips

### 1. Enable Caching

The plugin automatically caches API responses for 5 minutes.

### 2. Lazy Load

Widgets automatically lazy-load images for better performance.

### 3. Preconnect

Add to your theme's `header.php`:

```html
<link rel="preconnect" href="https://cdn.grooveshop.com">
<link rel="preconnect" href="https://api.grooveshop.com">
```

### 4. Limit Widget Count

Don't add too many widgets on one page. 2-3 is ideal.

---

## Advanced

### Custom Product Click Tracking

```php
// Add to functions.php
add_action('wp_footer', 'custom_tracking');
function custom_tracking() {
  ?>
  <script>
    GrooveShopRecommendations.on('click', function(event) {
      // Send to Google Analytics
      gtag('event', 'recommendation_click', {
        product_id: event.productId,
        position: event.position
      });
    });
  </script>
  <?php
}
```

### Sync All Products Manually

```php
// Add to functions.php (temporary - remove after running once)
add_action('init', 'sync_all_products');
function sync_all_products() {
  if (!isset($_GET['sync_products'])) return;

  $products = wc_get_products(['limit' => -1]);

  foreach ($products as $product) {
    GrooveShop_WooCommerce::sync_product($product->get_id());
  }

  wp_die('Products synced!');
}
```

Then visit: `yoursite.com/?sync_products`

---

## FAQ

**Q: Is the plugin free?**
A: Yes! The plugin is 100% free. GrooveShop offers a free tier with 10,000 monthly impressions.

**Q: Will this slow down my site?**
A: No! The widget is only 8.8 KB gzipped and loads asynchronously.

**Q: Does it work with page builders?**
A: Yes! Works with Elementor, Divi, Beaver Builder, and all major page builders.

**Q: Can I customize the appearance?**
A: Yes! Full customization via CSS custom properties.

**Q: Does it work with caching plugins?**
A: Yes! Works with WP Rocket, W3 Total Cache, and all major caching plugins.

**Q: What about GDPR?**
A: The plugin is GDPR compliant. No personal data is collected without consent.

---

## Support

- **Documentation**: [docs.grooveshop.com](https://docs.grooveshop.com)
- **Support Forum**: [wordpress.org/support/plugin/grooveshop-recommendations](https://wordpress.org/support/plugin/grooveshop-recommendations)
- **Email**: support@grooveshop.com
- **Dashboard**: [dashboard.grooveshop.com](https://dashboard.grooveshop.com)

---

## See Also

- [Shopify Integration](./shopify.md)
- [Quick Start Guide](../guides/quick-start.md)
- [Customization Guide](../guides/theming.md)
- [Code Recipes](../examples/recipes.md)
