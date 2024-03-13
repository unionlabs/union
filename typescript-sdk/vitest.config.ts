import { defineConfig } from "vitest/config"

export default defineConfig({
  resolve: {
    alias: {
      "#": "./src"
    }
  },
  test: {}
})
