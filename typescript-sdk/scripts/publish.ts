#!/usr/bin/env bun
import * as Bun from "bun"
import { parseArgs } from "node:util"
import { consola } from "./logger.ts"
import jsrJson from "../jsr.json" with { type: "json" }
import packageJson from "../package.json" with { type: "json" }

const CURRENT_JSR_JSON_VERSION = jsrJson.version
const CURRENT_PACKAGE_JSON_VERSION = packageJson.version
/**
 * Use this script to publish a new version of the TypeScript SDK to JSR registry
 * This will check if the contracts in the SDK are up to date with the contracts in the registry
 * If not it will fail
*
* Usage:
*
* `bun scripts/publish --period patch`

* `bun scripts/publish --period minor --dry-run`
*/

const { values } = parseArgs({
  args: process.argv.slice(2),
  strict: true,
  options: {
    period: { type: "string", default: "patch" },
    "dry-run": { type: "boolean", default: false }
  }
})

const PERIOD = values.period ?? "patch"
const DRY_RUN = values["dry-run"] ?? false

main().catch(_ => {
  consola.error(_)
  process.exit(1)
})

async function main() {
  try {
    if (DRY_RUN) {
      return await Bun.$ /* sh */`bunx jsr publish --allow-dirty --allow-slow-types --dry-run`
    }

    const bumpPackage = await Bun.$ /* sh */`npm version --preiod ${PERIOD} --no-git-tag-version`

    const version = bumpPackage.text().trim().replace(/^v/, "")

    // sync jsr.json version with package.json version
    const syncJsr =
      await Bun.$ /* sh */`jq --arg version "${version}" '.version = $version' jsr.json > jsr.temp.json && mv jsr.temp.json jsr.json`

    consola.info("Sync jsr.json version with package.json version", syncJsr.text())

    return await Bun.$ /* sh */`bunx jsr publish --allow-dirty --allow-slow-types`
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    consola.error(errorMessage)

    // revert changes
    await resetVersions()
    consola.info("Reset package.json version")
  }
}

async function resetVersions() {
  const currentVersion = await Bun.$ /* sh */`jq --raw-output .version package.json`.text()
  const newVersion =
    await Bun.$ /* sh */`${currentVersion.trim()} | awk -F'[.-]' '{print $1"."$2"."$3"-"$4"."$6-1}'`
  await Bun.$ /* sh */`npm version ${newVersion} --no-git-tag-version`
}
