import adapter from '@sveltejs/adapter-static'
import { preprocessMeltUI, sequence } from '@melt-ui/pp'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: sequence([
    vitePreprocess(),
    // has to be last
    preprocessMeltUI()
  ]),
  kit: {
    csp: {
      directives: {
        'frame-ancestors': [
          'self',
          'localhost',
          'localhost:*',
          'https://verify.walletconnect.com/',
          'https://verify.walletconnect.org/',
          'https://*.union.build',
          'https://union.build'
        ]
      }
    },
    adapter: adapter({
      strict: true
    }),
    /** @note `$` is a svelte path alias convention */
    alias: {
      $: './src/',
      $styles: './src/styles'
    }
  }
}
