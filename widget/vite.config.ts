import { defineConfig } from 'vite';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';

export default defineConfig({
  plugins: [
    dts({
      insertTypesEntry: true,
    }),
  ],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'GrooveShopRecommendations',
      formats: ['es', 'umd'],
      fileName: (format) => `widget.${format}.js`,
    },
    rollupOptions: {
      output: {
        assetFileNames: 'widget.[ext]',
      },
    },
    sourcemap: true,
    minify: 'esbuild',
  },
  server: {
    port: 3001,
    open: '/demo.html',
  },
});
