import * as Glob from "glob"
import * as Fs from "node:fs"

const dirs = [...Glob.sync("ts-sdk-*/*/")]
dirs.forEach((pkg) => {
  const files = [".tsbuildinfo", "docs", "build", "dist", "coverage"]

  files.forEach((file) => {
    if (pkg === "." && file === "docs") {
      return
    }

    Fs.rmSync(`${pkg}/${file}`, { recursive: true, force: true }, () => {})
  })
})
