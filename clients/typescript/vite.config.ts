import { defineConfig } from 'vite';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';

export default defineConfig({
  plugins: [dts({ rollupTypes: true })],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'RecommendationEngineClient',
      fileName: 'index',
      formats: ['es'],
    },
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: true,
    rollupOptions: {
      external: [],
      output: {
        preserveModules: false,
      },
    },
  },
  resolve: {
    extensions: ['.ts', '.js'],
  },
});
