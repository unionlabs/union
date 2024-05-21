import preprocess from "svelte-preprocess"
import childProcess from "node:child_process"
import adapterStatic from "@sveltejs/adapter-static"
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: [
    // https://kit.svelte.dev/docs/integrations#preprocessors-svelte-preprocess
    vitePreprocess(),
    preprocess({
      postcss: {
        configFilePath: "./postcss.config.cjs"
      }
    })
  ],
  kit: {
    // https://kit.svelte.dev/docs/adapter-static
    adapter: adapterStatic({
      strict: true,
      pages: "build",
      assets: "build",
      fallback: "index.html"
    }),
    version: {
      // derterministic build version
      name: getVersion({ short: true })
    },
    /** @note `$` is a svelte path alias convention */
    alias: {
      $: "./src/",
      $styles: "./src/styles"
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
