import { defineConfig } from "vitest/config"
import tsconfigPaths from "vite-tsconfig-paths"

export default defineConfig({
  test: {
    onConsoleLog(log: string, type: "stdout" | "stderr"): boolean | undefined {
      return !(log === "message from third party library" && type === "stdout")
    }
  },
  resolve: {},
  plugins: [tsconfigPaths()]
})
