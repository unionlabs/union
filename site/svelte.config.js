import url from 'node:url'
import path from 'node:path'
import { mdsvex } from 'mdsvex'
import preprocess from 'svelte-preprocess'
import adapter from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/kit/vite'

const __dirname = path.dirname(url.fileURLToPath(import.meta.url))

/** @type {import('@sveltejs/kit').Config} */
export default {
  extensions: ['.svelte', '.svx', '.md'],
  preprocess: [
    mdsvex({
      extensions: ['.svx', '.md'],
      layout: {
        blog: path.join(__dirname, './src/mdsvex/BlogLayout.svelte'),
      },
    }),
    vitePreprocess(),
    preprocess({
      postcss: {
        configFilePath: path.resolve(__dirname, './postcss.config.cjs'),
      },
    }),
  ],

  kit: {
    adapter: adapter(),
  },
}
