/**
 * CAUTION: these utilities require Node.js to run
 */

import fs from "node:fs/promises"
import path from "node:path"
import url from "node:url"

const __dirname = path.dirname(url.fileURLToPath(import.meta.url))

export function getFileContent({ filepath }: { filepath: string }) {
  const file = path.resolve(__dirname, filepath)
  if (!fs.stat(file)) {
    throw new Error(`File not found: ${file}`)
  }
  return fs.readFile(file, { encoding: "utf8" })
}
