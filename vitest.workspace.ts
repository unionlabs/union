import { defineWorkspace } from "vitest/config"

export default defineWorkspace([
  "ts-sdk/vitest.config.ts",
  "ts-sdk-evm/vitest.config.ts",
  "ts-sdk-cosmos/vitest.config.ts",
  "effect-svelte/vitest.config.ts",
])
