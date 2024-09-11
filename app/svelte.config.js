import path from "node:path"
import { sveltePreprocess } from "svelte-preprocess"
import adapterStatic from "@sveltejs/adapter-static"
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"
import svelteReactPreprocess from "svelte-preprocess-react/preprocessReact"

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: [
    // https://kit.svelte.dev/docs/integrations#preprocessors-svelte-preprocess
    vitePreprocess(),
    sveltePreprocess({
      postcss: {
        configFilePath: "./postcss.config.cjs"
      }
    }),
    svelteReactPreprocess()
  ],
  vitePlugin: {
    experimental: {},
    inspector: {
      holdMode: true,
      showToggleButton: "active",
      toggleKeyCombo: "control-shift",
      toggleButtonPos: "bottom-right"
    }
  },
  kit: {
    // https://kit.svelte.dev/docs/adapter-static
    adapter: adapterStatic({
      strict: true,
      pages: "build",
      assets: "build",
      fallback: "index.html"
    }),
    /** @note `$` is a svelte path alias convention */
    alias: {
      $: path.resolve("./src/"),
      $styles: path.resolve("./src/styles"),
      "~static": path.resolve("./static/")
    }
  }
}
