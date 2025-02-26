import adapterStatic from "@sveltejs/adapter-static"
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://svelte.dev/docs/kit/integrations
  // for more information about preprocessors
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapterStatic({
      strict: true,
      pages: "build",
      assets: "build",
      fallback: "index.html"
    })
  }
}

export default config
