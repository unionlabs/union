import url from 'node:url'
import path from 'node:path'
import preprocess from 'svelte-preprocess'
import adapterAuto from '@sveltejs/adapter-auto'
import adapterStatic from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

const isStatic = !!process.env.SVELTE_BUILD_STATIC

const __dirname = path.dirname(url.fileURLToPath(import.meta.url))

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: [
    vitePreprocess(),
    preprocess({
      postcss: {
        configFilePath: path.resolve(__dirname, './postcss.config.cjs'),
      },
    }),
  ],
  kit: {
    adapter: adapterAuto(),
  },
}
