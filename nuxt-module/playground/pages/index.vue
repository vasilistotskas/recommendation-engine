<template>
  <div class="container">
    <h1>GrooveShop Recommendations - Nuxt Module Demo</h1>

    <section class="demo-section">
      <h2>Trending Products (Carousel)</h2>
      <GrooveshopCarousel
        type="trending"
        :count="5"
      />
    </section>

    <section class="demo-section">
      <h2>Similar Products (Grid)</h2>
      <GrooveshopGrid
        type="similar"
        product-id="123"
        :count="8"
      />
    </section>

    <section class="demo-section">
      <h2>Recently Viewed (List)</h2>
      <GrooveshopList
        type="recently-viewed"
        :count="6"
      />
    </section>

    <section class="demo-section">
      <h2>Composable Demo</h2>
      <div class="composable-demo">
        <p><strong>Widget Ready:</strong> {{ isReady }}</p>
        <button @click="handleRefresh">Refresh Widgets</button>
        <button @click="handleTrack">Track Test Event</button>
        <button @click="handleSetUser">Set Test User</button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
const { isReady, trackEvent, setUser, refresh, on } = useGrooveshop()

// Listen for clicks
on('click', (event) => {
  console.log('Product clicked:', event)
})

const handleRefresh = async () => {
  console.log('Refreshing widgets...')
  await refresh()
  console.log('Widgets refreshed!')
}

const handleTrack = () => {
  trackEvent('view', {
    productId: '123',
    userId: 'test-user'
  })
  console.log('Event tracked!')
}

const handleSetUser = () => {
  setUser({
    id: 'test-user-456',
    email: 'test@example.com',
    name: 'Test User'
  })
  console.log('User set!')
}
</script>

<style scoped>
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

h1 {
  font-size: 2.5rem;
  margin-bottom: 2rem;
  color: #333;
}

.demo-section {
  margin: 3rem 0;
}

.demo-section h2 {
  font-size: 1.5rem;
  margin-bottom: 1rem;
  color: #555;
}

.composable-demo {
  background: #f5f5f5;
  padding: 1.5rem;
  border-radius: 8px;
}

.composable-demo p {
  margin-bottom: 1rem;
}

.composable-demo button {
  margin-right: 1rem;
  padding: 0.5rem 1rem;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.composable-demo button:hover {
  background: #0056b3;
}
</style>
