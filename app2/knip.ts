import type { KnipConfig } from "knip"

const config: KnipConfig = {
  svelte: {
    entry: [
      "svelte.config.js",
      "vite.config.{js,mjs,ts,cjs,mts,cts}",
      "src/routes/**/+{page,server,page.server,error,layout,layout.server}{,@*}.{js,ts,svelte}",
      "src/hooks.{server,client}.{js,ts}",
      "src/params/*.{js,ts}",
    ],
  },
  project: [
    "src/lib/**/*.{js,ts,svelte}",
    "!src/generated/**/*",
  ],
  ignoreBinaries: [
    "vitest",
    "supabase",
  ],
  ignoreDependencies: [
    "@unionlabs/client",
    "@unionlabs/sdk",
    "@unionlabs/sdk-evm",
    "@unionlabs/sdk-cosmos",
    "@safe-global/safe-gateway-typescript-sdk",
    "@web3modal/wagmi",
    "tailwindcss",
    "vitest",
    "@effect/vitest",
  ],
  ignoreUnresolved: [
    /^\$env\//,
  ],
  // ignoreUnresolved: ['some-virtual-import$'],
  compilers: { svelte: true },
}

export default config
