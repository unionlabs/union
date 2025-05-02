import tsconfigPaths from "vite-tsconfig-paths"
import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    onConsoleLog(log: string, type: "stdout" | "stderr"): boolean | undefined {
      return !(log === "message from third party library" && type === "stdout")
    },
  },
  resolve: {},
  plugins: [tsconfigPaths()],
})
