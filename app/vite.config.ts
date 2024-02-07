import { defineConfig } from 'vitest/config'
import { sveltekit } from '@sveltejs/kit/vite'

export default defineConfig({
  plugins: [sveltekit()],
  // Node polyfills
  resolve: {
    alias: {
      'node:buffer': 'buffer'
    }
  },
  server: {
    port: Number(process.env.PORT || 5173),
    /**
     * (WalletConnect 2.0)
     * [vite] failed to connect to websocket.
     * Check out your Vite / network configuration and https://vitejs.dev/config/server-options.html#server-hmr .
     */
    hmr: { overlay: false }
  },
  test: { include: ['src/**/*.{test,spec}.{js,ts}'] }
})
