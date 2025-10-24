import { json } from '@remix-run/node';
import { authenticate } from '../shopify.server';
import prisma from '../server/db.server';

export const action = async ({ request }: { request: Request }) => {
  const { topic, shop, session, admin, payload } = await authenticate.webhook(request);

  console.log(`Received ${topic} webhook for ${shop}`);

  switch (topic) {
    case 'APP_UNINSTALLED':
      // Deactivate store
      await prisma.store.update({
        where: { shop },
        data: { isActive: false },
      });
      break;

    case 'PRODUCTS_CREATE':
    case 'PRODUCTS_UPDATE':
      await syncProduct(shop, payload);
      break;

    case 'PRODUCTS_DELETE':
      await deleteProduct(shop, payload);
      break;

    case 'ORDERS_CREATE':
      await trackOrder(shop, payload);
      break;

    default:
      console.log(`Unhandled webhook topic: ${topic}`);
  }

  return json({ success: true });
};

async function syncProduct(shop: string, product: any) {
  const store = await prisma.store.findUnique({ where: { shop } });
  if (!store || !store.isActive) return;

  const { apiKey, tenantId, settings } = store;
  const apiUrl = (settings as any)?.apiUrl || 'https://api.grooveshop.com';

  // Prepare product data
  const productData = {
    entity_id: `shopify_${product.id}`,
    attributes: {
      name: product.title,
      price: parseFloat(product.variants[0]?.price || 0),
      image_url: product.images[0]?.src || '',
      category: product.product_type || '',
      vendor: product.vendor || '',
      tags: product.tags ? product.tags.split(', ') : [],
      url: `https://${shop}/products/${product.handle}`,
      inventory: product.variants.reduce((sum: number, v: any) => sum + (v.inventory_quantity || 0), 0),
      rating: 0, // Shopify doesn't provide ratings by default
    },
  };

  // Send to GrooveShop API
  try {
    const response = await fetch(`${apiUrl}/api/v1/products/${tenantId}`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(productData),
    });

    if (response.ok) {
      // Track sync in database
      await prisma.product.upsert({
        where: {
          shop_shopifyId: {
            shop,
            shopifyId: String(product.id),
          },
        },
        create: {
          shop,
          shopifyId: String(product.id),
          entityId: `shopify_${product.id}`,
          synced: true,
          lastSyncAt: new Date(),
        },
        update: {
          entityId: `shopify_${product.id}`,
          synced: true,
          lastSyncAt: new Date(),
        },
      });
    }
  } catch (error) {
    console.error('Error syncing product:', error);
  }
}

async function deleteProduct(shop: string, product: any) {
  const store = await prisma.store.findUnique({ where: { shop } });
  if (!store || !store.isActive) return;

  const { apiKey, tenantId, settings } = store;
  const apiUrl = (settings as any)?.apiUrl || 'https://api.grooveshop.com';

  try {
    await fetch(`${apiUrl}/api/v1/products/${tenantId}/shopify_${product.id}`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
      },
    });

    // Remove from database
    await prisma.product.deleteMany({
      where: {
        shop,
        shopifyId: String(product.id),
      },
    });
  } catch (error) {
    console.error('Error deleting product:', error);
  }
}

async function trackOrder(shop: string, order: any) {
  const store = await prisma.store.findUnique({ where: { shop } });
  if (!store || !store.isActive) return;

  const { apiKey, tenantId, settings } = store;
  const apiUrl = (settings as any)?.apiUrl || 'https://api.grooveshop.com';

  // Prepare order data
  const items = order.line_items.map((item: any) => ({
    entity_id: `shopify_${item.product_id}`,
    quantity: item.quantity,
    price: parseFloat(item.price),
  }));

  const orderData = {
    tenant_id: tenantId,
    user_id: order.customer?.id ? `shopify_customer_${order.customer.id}` : `shopify_guest_${order.email}`,
    order_id: `shopify_order_${order.id}`,
    items,
    total: parseFloat(order.total_price),
    timestamp: order.created_at,
  };

  try {
    const response = await fetch(`${apiUrl}/api/v1/interactions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(orderData),
    });

    if (response.ok) {
      await prisma.order.create({
        data: {
          shop,
          shopifyOrderId: String(order.id),
          tracked: true,
          trackedAt: new Date(),
        },
      });
    }
  } catch (error) {
    console.error('Error tracking order:', error);
  }
}
