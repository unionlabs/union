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
      fallback: "index.html"
    }),
    alias: {
      "@unionlabs/client": "../typescript-sdk/src/mod.js",
      "@unionlabs/sdk": "../ts-sdk/src/index.js",
      "@unionlabs/sdk/*": "../ts-sdk/src/*"
    }
  }
}

export default config
