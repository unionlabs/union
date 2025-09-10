import * as Glob from "glob"
import * as Fs from "node:fs"

const dirs = [
  ...Glob.sync("ts-sdk/*/"),
  ...Glob.sync("ts-sdk-evm/*/"),
  ...Glob.sync("ts-sdk-cosmos/*/"),
]
dirs.forEach((pkg) => {
  const files = [".tsbuildinfo", "docs", "build", "dist", "coverage"]

  files.forEach((file) => {
    if (pkg === "." && file === "docs") {
      return
    }

    Fs.rmSync(`${pkg}/${file}`, { recursive: true, force: true }, () => {})
  })
})
