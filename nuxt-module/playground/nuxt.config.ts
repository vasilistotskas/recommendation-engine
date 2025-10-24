export default defineNuxtConfig({
  modules: ['../src/module'],

  grooveshop: {
    apiKey: 'pk_test_demo_key_for_testing',
    tenantId: 'demo-store',
    debug: true
  },

  devtools: { enabled: true },

  compatibilityDate: '2024-11-01'
})
