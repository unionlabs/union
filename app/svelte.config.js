import adapter from '@sveltejs/adapter-cloudflare'
import { preprocessMeltUI, sequence } from '@melt-ui/pp'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: sequence([
    vitePreprocess(),
    preprocessMeltUI() // has to be last
  ]),
  kit: {
    adapter: adapter(),
    /** @note `$` is a svelte path alias convention */
    alias: {
      $: './src/',
      $styles: './src/styles'
    }
  }
}
