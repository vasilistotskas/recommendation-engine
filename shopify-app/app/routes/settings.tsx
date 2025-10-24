import { useState, useCallback } from 'react';
import { json, redirect } from '@remix-run/node';
import { useLoaderData, useSubmit, useNavigation } from '@remix-run/react';
import {
  Page,
  Layout,
  Card,
  FormLayout,
  TextField,
  Select,
  Checkbox,
  Button,
  BlockStack,
  Banner,
  Text,
} from '@shopify/polaris';
import { TitleBar } from '@shopify/app-bridge-react';
import { authenticate } from '../shopify.server';
import prisma from '../server/db.server';

export const loader = async ({ request }: { request: Request }) => {
  const { session } = await authenticate.admin(request);

  const store = await prisma.store.findUnique({
    where: { shop: session.shop },
  });

  return json({
    shop: session.shop,
    settings: store || {
      apiKey: '',
      tenantId: '',
      settings: {
        apiUrl: 'https://api.grooveshop.com',
        enableProductPage: true,
        enableCartPage: true,
        layout: 'carousel',
        count: 5,
        theme: 'light',
        autoTrack: true,
      },
    },
  });
};

export const action = async ({ request }: { request: Request }) => {
  const { session } = await authenticate.admin(request);
  const formData = await request.formData();

  const apiKey = formData.get('apiKey') as string;
  const tenantId = formData.get('tenantId') as string;
  const apiUrl = formData.get('apiUrl') as string;
  const enableProductPage = formData.get('enableProductPage') === 'true';
  const enableCartPage = formData.get('enableCartPage') === 'true';
  const layout = formData.get('layout') as string;
  const count = parseInt(formData.get('count') as string);
  const theme = formData.get('theme') as string;
  const autoTrack = formData.get('autoTrack') === 'true';

  // Validate credentials with GrooveShop API
  const response = await fetch(`${apiUrl}/api/v1/recommendations/${tenantId}/trending?count=1`, {
    headers: {
      Authorization: `Bearer ${apiKey}`,
    },
  });

  if (!response.ok) {
    return json({ error: 'Invalid API credentials' }, { status: 400 });
  }

  // Save settings
  await prisma.store.upsert({
    where: { shop: session.shop },
    create: {
      shop: session.shop,
      apiKey,
      tenantId,
      settings: {
        apiUrl,
        enableProductPage,
        enableCartPage,
        layout,
        count,
        theme,
        autoTrack,
      },
    },
    update: {
      apiKey,
      tenantId,
      settings: {
        apiUrl,
        enableProductPage,
        enableCartPage,
        layout,
        count,
        theme,
        autoTrack,
      },
      isActive: true,
    },
  });

  return redirect('/?saved=true');
};

export default function Settings() {
  const { shop, settings } = useLoaderData<typeof loader>();
  const submit = useSubmit();
  const navigation = useNavigation();

  const [apiKey, setApiKey] = useState(settings.apiKey || '');
  const [tenantId, setTenantId] = useState(settings.tenantId || '');
  const [apiUrl, setApiUrl] = useState(settings.settings?.apiUrl || 'https://api.grooveshop.com');
  const [enableProductPage, setEnableProductPage] = useState(settings.settings?.enableProductPage ?? true);
  const [enableCartPage, setEnableCartPage] = useState(settings.settings?.enableCartPage ?? true);
  const [layout, setLayout] = useState(settings.settings?.layout || 'carousel');
  const [count, setCount] = useState(String(settings.settings?.count || 5));
  const [theme, setTheme] = useState(settings.settings?.theme || 'light');
  const [autoTrack, setAutoTrack] = useState(settings.settings?.autoTrack ?? true);

  const isLoading = navigation.state === 'submitting';

  const handleSubmit = useCallback(() => {
    const formData = new FormData();
    formData.append('apiKey', apiKey);
    formData.append('tenantId', tenantId);
    formData.append('apiUrl', apiUrl);
    formData.append('enableProductPage', String(enableProductPage));
    formData.append('enableCartPage', String(enableCartPage));
    formData.append('layout', layout);
    formData.append('count', count);
    formData.append('theme', theme);
    formData.append('autoTrack', String(autoTrack));

    submit(formData, { method: 'post' });
  }, [apiKey, tenantId, apiUrl, enableProductPage, enableCartPage, layout, count, theme, autoTrack, submit]);

  return (
    <Page>
      <TitleBar title="Settings" />
      <BlockStack gap="500">
        <Layout>
          <Layout.Section>
            <Card>
              <BlockStack gap="400">
                <Text as="h2" variant="headingMd">
                  API Settings
                </Text>
                <Text as="p" variant="bodyMd">
                  Enter your GrooveShop API credentials. Get your API key from{' '}
                  <a href="https://dashboard.grooveshop.com" target="_blank" rel="noopener noreferrer">
                    your dashboard
                  </a>
                  .
                </Text>

                <FormLayout>
                  <TextField
                    label="API Key"
                    value={apiKey}
                    onChange={setApiKey}
                    autoComplete="off"
                    helpText="Your public API key (starts with pk_)"
                    requiredIndicator
                  />

                  <TextField
                    label="Tenant ID"
                    value={tenantId}
                    onChange={setTenantId}
                    autoComplete="off"
                    helpText="Your store identifier"
                    requiredIndicator
                  />

                  <TextField
                    label="API URL"
                    value={apiUrl}
                    onChange={setApiUrl}
                    autoComplete="off"
                    helpText="Leave default unless using custom endpoint"
                  />
                </FormLayout>
              </BlockStack>
            </Card>

            <Card>
              <BlockStack gap="400">
                <Text as="h2" variant="headingMd">
                  Widget Settings
                </Text>

                <FormLayout>
                  <Checkbox
                    label="Show similar products on product pages"
                    checked={enableProductPage}
                    onChange={setEnableProductPage}
                  />

                  <Checkbox
                    label="Show bundles on cart page"
                    checked={enableCartPage}
                    onChange={setEnableCartPage}
                  />

                  <Select
                    label="Default Layout"
                    options={[
                      { label: 'Carousel', value: 'carousel' },
                      { label: 'Grid', value: 'grid' },
                      { label: 'List', value: 'list' },
                    ]}
                    value={layout}
                    onChange={setLayout}
                  />

                  <TextField
                    label="Products to Show"
                    type="number"
                    value={count}
                    onChange={setCount}
                    min={1}
                    max={20}
                    autoComplete="off"
                    helpText="Number of products to display (1-20)"
                  />

                  <Select
                    label="Theme"
                    options={[
                      { label: 'Light', value: 'light' },
                      { label: 'Dark', value: 'dark' },
                      { label: 'Minimal', value: 'minimal' },
                    ]}
                    value={theme}
                    onChange={setTheme}
                  />

                  <Checkbox
                    label="Automatically track clicks and impressions"
                    checked={autoTrack}
                    onChange={setAutoTrack}
                  />
                </FormLayout>
              </BlockStack>
            </Card>

            <Button
              variant="primary"
              onClick={handleSubmit}
              loading={isLoading}
              disabled={!apiKey || !tenantId}
            >
              Save Settings
            </Button>
          </Layout.Section>

          <Layout.Section variant="oneThird">
            <Card>
              <BlockStack gap="300">
                <Text as="h3" variant="headingMd">
                  Getting Started
                </Text>
                <ol style={{ paddingLeft: '20px', margin: 0 }}>
                  <li>Enter your API credentials</li>
                  <li>Configure widget settings</li>
                  <li>Enable auto-placement or use theme extension</li>
                </ol>
              </BlockStack>
            </Card>

            <Card>
              <BlockStack gap="300">
                <Text as="h3" variant="headingMd">
                  Need Help?
                </Text>
                <Button url="https://docs.grooveshop.com/shopify" external>
                  Documentation
                </Button>
                <Button url="https://support.grooveshop.com" external>
                  Support
                </Button>
              </BlockStack>
            </Card>
          </Layout.Section>
        </Layout>
      </BlockStack>
    </Page>
  );
}
