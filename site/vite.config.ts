import { defineConfig } from 'vite'
import { sveltekit } from '@sveltejs/kit/vite'

export default defineConfig({
  plugins: [sveltekit()],
  // Node polyfills
  // define: {
  //   'process.env': {},
  // },
  // // Node polyfills
  // resolve: {
  //   alias: {
  //     stream: 'rollup-plugin-node-polyfills/polyfills/stream',
  //     'node:buffer': 'buffer',
  //   },
  // },
  // // ECMAScript shims
  // build: {
  //   rollupOptions: {
  //     external: ['array.prototype.group', 'array.prototype.grouptomap'],
  //   },
  // },
  // // (WalletConnect 2.0)
  // // [vite] failed to connect to websocket.
  // // Check out your Vite / network configuration and https://vitejs.dev/config/server-options.html#server-hmr .
  // server: {
  //   hmr: { overlay: false },
  // },
})
