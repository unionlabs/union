import * as path from "node:path"
import type { ViteUserConfig } from "vitest/config"

const config: ViteUserConfig = {
  esbuild: {
    target: "es2020"
  },
  test: {
    setupFiles: [path.join(__dirname, "vitest.setup.ts")],
    fakeTimers: {
      toFake: undefined
    },
    sequence: {
      concurrent: true
    },
    include: ["test/**/*.test.ts"],
    alias: {
      ["@unionlabs/sdk/test"]: path.join(__dirname, "ts-sdk", "test"),
      ["@unionlabs/sdk"]: path.join(__dirname, "ts-sdk", "src")
    }
  }
}

export default config
