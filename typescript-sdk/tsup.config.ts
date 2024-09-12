import { defineConfig } from "tsup"

export default defineConfig({
  dts: true,
  clean: true,
  shims: true,
  format: ["esm"],
  treeshake: true,
  target: "es2022",
  entry: { index: "./src/mod.ts" },
  outExtension: _context => ({ js: ".mjs", dts: ".d.ts" })
})
