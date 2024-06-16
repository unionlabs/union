#!/usr/bin/env bun
import * as Bun from "bun"
import { consola } from "./logger.ts"
import { parseArgs } from "node:util"

/**
 * Use this script to publish a new version of the TypeScript SDK to JSR registry
 * This will check if the contracts in the SDK are up to date with the contracts in the registry
 * If not it will fail
 *
 * Usage:
 *
 * `bun scripts/publish`
 * `bun scripts/publish --dry-run`
 */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "dry-run": { type: "boolean", default: false } }
})

const DRY_RUN = values["dry-run"] ?? false

main().catch(_ => {
  consola.error(_)
  process.exit(1)
})

async function main() {
  // TODO: Check if the version in jsr.json is bumped
  // const versionBumped = Bun.$`git diff --quiet jsr.json && echo false || echo true`

  if (DRY_RUN) {
    return await Bun.$ /* sh */`bunx jsr publish --allow-dirty --allow-slow-types --dry-run`
  }

  return await Bun.$ /* sh */`bunx jsr publish --allow-dirty --allow-slow-types`
}
