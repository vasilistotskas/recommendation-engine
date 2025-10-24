import { useEffect } from 'react';
import { json } from '@remix-run/node';
import { useLoaderData, useNavigate } from '@remix-run/react';
import { Page, Layout, Card, Text, BlockStack, Button, Banner, List } from '@shopify/polaris';
import { TitleBar } from '@shopify/app-bridge-react';
import { authenticate } from '../shopify.server';

export const loader = async ({ request }: { request: Request }) => {
  const { admin, session } = await authenticate.admin(request);

  // Get store settings from database
  const store = await prisma.store.findUnique({
    where: { shop: session.shop },
  });

  return json({
    shop: session.shop,
    isConfigured: !!store?.apiKey,
    settings: store?.settings || {},
  });
};

export default function Index() {
  const { shop, isConfigured, settings } = useLoaderData<typeof loader>();
  const navigate = useNavigate();

  return (
    <Page>
      <TitleBar title="GrooveShop Recommendations" />
      <BlockStack gap="500">
        {!isConfigured && (
          <Banner
            title="Welcome to GrooveShop Recommendations!"
            status="info"
            action={{ content: 'Configure Now', onAction: () => navigate('/settings') }}
          >
            <p>
              Get started by configuring your API credentials and widget settings.
            </p>
          </Banner>
        )}

        <Layout>
          <Layout.Section>
            <Card>
              <BlockStack gap="400">
                <Text as="h2" variant="headingMd">
                  AI-Powered Product Recommendations
                </Text>
                <Text as="p" variant="bodyMd">
                  Increase your store's revenue with intelligent product recommendations
                  powered by machine learning.
                </Text>
                <List type="bullet">
                  <List.Item>Similar products on product pages</List.Item>
                  <List.Item>Trending products across your store</List.Item>
                  <List.Item>Personalized recommendations for each customer</List.Item>
                  <List.Item>Smart bundles and complementary products</List.Item>
                  <List.Item>Real-time social proof and updates</List.Item>
                </List>
                <Button
                  variant="primary"
                  onClick={() => navigate('/settings')}
                >
                  {isConfigured ? 'Manage Settings' : 'Get Started'}
                </Button>
              </BlockStack>
            </Card>
          </Layout.Section>

          <Layout.Section variant="oneThird">
            <Card>
              <BlockStack gap="300">
                <Text as="h3" variant="headingMd">
                  Quick Stats
                </Text>
                <BlockStack gap="200">
                  <Text as="p" variant="bodyMd">
                    <strong>Status:</strong> {isConfigured ? 'Active' : 'Not Configured'}
                  </Text>
                  <Text as="p" variant="bodyMd">
                    <strong>Store:</strong> {shop}
                  </Text>
                </BlockStack>
                <Button
                  url="https://docs.grooveshop.com/shopify"
                  external
                >
                  View Documentation
                </Button>
              </BlockStack>
            </Card>

            <Card>
              <BlockStack gap="300">
                <Text as="h3" variant="headingMd">
                  Need Help?
                </Text>
                <Button
                  url="https://support.grooveshop.com"
                  external
                >
                  Contact Support
                </Button>
                <Button
                  url="https://dashboard.grooveshop.com"
                  external
                >
                  View Analytics
                </Button>
              </BlockStack>
            </Card>
          </Layout.Section>
        </Layout>
      </BlockStack>
    </Page>
  );
}
