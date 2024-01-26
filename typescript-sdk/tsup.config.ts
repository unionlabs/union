import { defineConfig } from 'tsup'

/**
 * @see https://tsup.egoist.dev/#usage
 */

export default defineConfig({
  dts: true,
  clean: true,
  // https://tsup.egoist.dev/#inject-cjs-and-esm-shims
  shims: true,
  bundle: true,
  outDir: 'dist',
  target: 'node20',
  platform: 'node',
  treeshake: 'recommended',
  entry: ['./src/index.ts'],
  format: ['esm', 'cjs']
})
