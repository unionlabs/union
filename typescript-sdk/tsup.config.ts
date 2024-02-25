import { defineConfig } from "tsup";

/**
 * @see https://tsup.egoist.dev/#usage
 */

export default defineConfig({
  dts: true,
  clean: true,
  // https://tsup.egoist.dev/#inject-cjs-and-esm-shims
  shims: true,
  bundle: true,
  outDir: "dist",
  format: ["esm"],
  target: "esnext",
  platform: "browser",
  treeshake: "recommended",
  entry: ["./src/index.ts"],
});
