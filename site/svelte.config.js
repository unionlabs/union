import { vitePreprocess } from "@astrojs/svelte"

/** @type {import('@sveltejs/vite-plugin-svelte').Options} */
export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    hydratable: true,
    dev: process.env.NODE_ENV !== "production"
  }
}
