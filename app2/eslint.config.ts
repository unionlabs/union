import { includeIgnoreFile } from "@eslint/compat"
import * as U from "@unionlabs/eslint-config"
import type { Linter } from "eslint"
import svelteConfig from "./svelte.config.js"

const gitignore = includeIgnoreFile(
  new URL(import.meta.resolve("./.gitignore")).pathname,
)

const config: Linter.Config[] = [
  gitignore,
  ...U.configs.svelte(svelteConfig),
]

export default config
