import adapter from "@sveltejs/adapter-static"
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),
    prerender: {
      handleUnseenRoutes: "ignore", // Dynamic routes like /[chainId] can't be prerendered
    },
    alias: {
      "@unionlabs/sdk": "../ts-sdk/src/index.js",
      "@unionlabs/sdk/*": "../ts-sdk/src/*",
      "@unionlabs/sdk-evm": "../ts-sdk-evm/src/index.js",
      "@unionlabs/sdk-evm/*": "../ts-sdk-evm/src/*",
      "@unionlabs/sdk-cosmos": "../ts-sdk-cosmos/src/index.js",
      "@unionlabs/sdk-cosmos/*": "../ts-sdk-cosmos/src/*",
      "@unionlabs/effect-svelte": "../effect-svelte/src/lib/index.js",
      "@unionlabs/effect-svelte/*": "../effect-svelte/src/lib/*",
    },
  },
}

export default config
