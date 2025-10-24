/**
 * GrooveShop Widget Loader for Shopify
 *
 * Loads the GrooveShop widget script and initializes it with store settings
 */

(function() {
  'use strict';

  // Get store settings from meta tags (injected by app)
  const apiKey = document.querySelector('meta[name="grooveshop-api-key"]')?.content;
  const tenantId = document.querySelector('meta[name="grooveshop-tenant-id"]')?.content;
  const apiUrl = document.querySelector('meta[name="grooveshop-api-url"]')?.content || 'https://api.grooveshop.com';

  if (!apiKey || !tenantId) {
    console.warn('GrooveShop: Missing API credentials');
    return;
  }

  // Load widget script
  const script = document.createElement('script');
  script.src = 'https://cdn.grooveshop.com/widget/v1/grooveshop-recommendations.js';
  script.async = true;
  script.defer = true;

  script.onload = function() {
    // Initialize widget with store settings
    if (window.GrooveShopRecommendations) {
      window.GrooveShopRecommendations.init({
        apiKey: apiKey,
        tenantId: tenantId,
        apiUrl: apiUrl,
        platform: 'shopify',
        debug: false,
      });
    }
  };

  script.onerror = function() {
    console.error('GrooveShop: Failed to load widget script');
  };

  // Inject script
  document.head.appendChild(script);

  // Track product views
  if (window.Shopify && window.Shopify.theme && window.Shopify.theme.name === 'product') {
    const productId = document.querySelector('[data-product-id]')?.dataset.productId;
    if (productId) {
      // Wait for widget to load
      const checkWidget = setInterval(function() {
        if (window.GrooveShopRecommendations && window.GrooveShopRecommendations.trackEvent) {
          clearInterval(checkWidget);
          window.GrooveShopRecommendations.trackEvent('view', {
            productId: 'shopify_' + productId,
          });
        }
      }, 100);
    }
  }
})();
