import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';

// Basic configuration
export default defineConfig({
  integrations: [svelte()],
  site: 'http://localhost:4321',
  vite: {
    optimizeDeps: {
      exclude: ['@splinetool/runtime', 'echarts'],
    },
    ssr: {
      noExternal: ['@splinetool/runtime', 'echarts'],
    }
  }
});
