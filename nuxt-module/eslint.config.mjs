// @ts-check
import { createConfigForNuxt } from '@nuxt/eslint-config/flat'

export default createConfigForNuxt({
  features: {
    tooling: true,
    stylistic: false
  }
})
  .append({
    files: ['**/*.ts', '**/*.mts', '**/*.cts', '**/*.vue'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-unused-vars': ['error', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^props$' // Allow unused 'props' in Vue components
      }],
      'vue/multi-word-component-names': 'off',
      'vue/require-default-prop': 'off'
    }
  })
  .append({
    ignores: [
      'dist',
      'node_modules',
      '.nuxt',
      '.output',
      'playground/.nuxt',
      'playground/.output'
    ]
  })
