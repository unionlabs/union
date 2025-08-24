/**
 * CAUTION: these utilities require Node.js to run
 */

import * as Fs from "node:fs/promises"
import * as Path from "node:path"
import * as Url from "node:url"

const __dirname = Path.dirname(Url.fileURLToPath(import.meta.url))

export function getFileContent({ filepath }: { filepath: string }) {
  const file = Path.resolve(__dirname, filepath)
  if (!Fs.stat(file)) {
    throw new Error(`File not found: ${file}`)
  }
  return Fs.readFile(file, { encoding: "utf8" })
}
