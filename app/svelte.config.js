import path from "node:path"
import childProcess from "node:child_process"
import { sveltePreprocess } from "svelte-preprocess"
import adapterStatic from "@sveltejs/adapter-static"
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: [
    // https://kit.svelte.dev/docs/integrations#preprocessors-svelte-preprocess
    vitePreprocess(),
    sveltePreprocess({
      postcss: {
        configFilePath: "./postcss.config.cjs"
      }
    })
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
    version: {
      // deterministic build version
      name: getVersion({ short: true })
    },
    /** @note `$` is a svelte path alias convention */
    alias: {
      $: path.resolve("./src/"),
      $styles: path.resolve("./src/styles")
    }
  }
}

function getVersion({ short = false } = {}) {
  try {
    const version = childProcess.execSync("git rev-parse HEAD").toString().trim()
    return short ? version.slice(0, 7) : version
  } catch (error) {
    const timestamp = Date.now().toString()
    console.error(
      `could not get commit-hash to set a version id, falling back on timestamp ${timestamp}`
    )
    return timestamp
  }
}
