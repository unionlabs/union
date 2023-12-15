import url from 'node:url'
import path from 'node:path'
import { mdsvex } from 'mdsvex'
import preprocess from 'svelte-preprocess'
import adapter from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

const __dirname = path.dirname(url.fileURLToPath(import.meta.url))

/** @type {import('@sveltejs/kit').Config} */
export default {
  extensions: ['.svelte', '.md', '.svx'],
  preprocess: [
    vitePreprocess(),
    preprocess({ postcss: true }),
    mdsvex({
      extensions: ['.md'],
      layout: {
        blog: path.join(__dirname, 'src/mdsvex/BlogLayout.svelte'),
      },
    }),
  ],

  kit: {
    adapter: adapter(),
  },
}
