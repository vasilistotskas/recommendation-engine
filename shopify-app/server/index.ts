import { createServer } from '@remix-run/node';
import { installGlobals } from '@remix-run/node';
import { shopifyApp, ApiVersion, AppDistribution, DeliveryMethod } from '@shopify/shopify-app-remix/server';
import { PrismaSessionStorage } from '@shopify/shopify-app-session-storage-prisma';
import { restResources } from '@shopify/shopify-api/rest/admin/2024-01';
import prisma from './db.server';

installGlobals();

const shopify = shopifyApp({
  apiKey: process.env.SHOPIFY_API_KEY!,
  apiSecretKey: process.env.SHOPIFY_API_SECRET || '',
  apiVersion: ApiVersion.January24,
  scopes: process.env.SCOPES?.split(',') || ['read_products', 'write_products', 'read_orders', 'write_script_tags', 'read_customers', 'write_themes'],
  appUrl: process.env.SHOPIFY_APP_URL || '',
  authPathPrefix: '/auth',
  sessionStorage: new PrismaSessionStorage(prisma),
  distribution: AppDistribution.AppStore,
  restResources,
  webhooks: {
    APP_UNINSTALLED: {
      deliveryMethod: DeliveryMethod.Http,
      callbackUrl: '/webhooks',
    },
    PRODUCTS_CREATE: {
      deliveryMethod: DeliveryMethod.Http,
      callbackUrl: '/webhooks',
    },
    PRODUCTS_UPDATE: {
      deliveryMethod: DeliveryMethod.Http,
      callbackUrl: '/webhooks',
    },
    PRODUCTS_DELETE: {
      deliveryMethod: DeliveryMethod.Http,
      callbackUrl: '/webhooks',
    },
    ORDERS_CREATE: {
      deliveryMethod: DeliveryMethod.Http,
      callbackUrl: '/webhooks',
    },
  },
  hooks: {
    afterAuth: async ({ session }) => {
      // Register webhooks after authentication
      shopify.registerWebhooks({ session });
    },
  },
  future: {
    v3_webhookAdminContext: true,
    v3_authenticatePublic: true,
  },
});

export default shopify;
export const authenticate = shopify.authenticate;
export const login = shopify.login;
export const sessionStorage = shopify.sessionStorage;
