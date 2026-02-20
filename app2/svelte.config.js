import adapter from "@sveltejs/adapter-static"
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  // TODO: hook me up to production mode
  // vitePlugin: {
  //   inspector: {
  //     toggleKeyCombo: "control-shift",
  //     holdMode: true,
  //     showToggleButton: "always",
  //     toggleButtonPos: "bottom-right"
  //   }
  // },
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),
    alias: {
      "@unionlabs/sdk": "../ts-sdk/src/index.js",
      "@unionlabs/sdk/*": "../ts-sdk/src/*",
      "@unionlabs/sdk-evm": "../ts-sdk-evm/src/index.js",
      "@unionlabs/sdk-evm/*": "../ts-sdk-evm/src/*",
      "@unionlabs/sdk-cosmos": "../ts-sdk-cosmos/src/index.js",
      "@unionlabs/sdk-cosmos/*": "../ts-sdk-cosmos/src/*",
      "@unionlabs/effect-svelte": "../effect-svelte/src/lib/index.js",
      "@unionlabs/effect-svelte/*": "../ts-sdk-cosmos/src/lib/*",
    },
  },
}

export default config
