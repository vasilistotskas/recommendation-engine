import { defineBuildConfig } from 'unbuild'

export default defineBuildConfig({
  entries: ['src/module'],
  clean: true,
  declaration: true,
  externals: ['@nuxt/kit', '@nuxt/schema', 'nuxt', 'defu'],
  rollup: {
    emitCJS: true
  }
})
