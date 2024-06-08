#!/usr/bin/env bun
import * as Bun from "bun"
import { UnionClient } from "#mod.ts"
import { consola } from "./logger.ts"
import { parseArgs } from "node:util"
import currentContracts_ from "~root/versions/contracts.json" with { type: "json" }

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

  /**
   * Compare contracts in the current version of the SDK with the up to date contracts in $REPO_ROOT/versions/contracts.json.
   */

  const { chainId, latest, ...currentContracts } = currentContracts_.find(
    c => c.latest === true
  ) as (typeof currentContracts_)[0]

  const pkgContracts = UnionClient.getContractAddresses()

  if (!Bun.deepEquals(currentContracts, pkgContracts, true)) {
    consola.fail("Contracts in the SDK are outdated:")
    consola.box(JSON.stringify({ currentContracts, pkgContracts }, undefined, 2))
  }

  if (DRY_RUN) return Bun.$ /* sh */`bunx jsr publish --allow-dirty --allow-slow-types --dry-run`

  return Bun.$ /* sh */`bunx jsr publish --allow-dirty --allow-slow-types`
}
