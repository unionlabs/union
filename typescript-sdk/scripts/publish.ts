#!/usr/bin/env bun
import * as Bun from "bun"
import { consola } from "./logger.ts"

/**
 * Use this script to publish a new version of the TypeScript SDK to JSR registry
 * This will check if the contracts in the SDK are up to date with the contracts in the registry
 * If not it will fail
 *
 * Usage:
 *
 * `bun scripts/publish`
 * `DRY_RUN=1 bun scripts/publish`
 */

const DRY_RUN = import.meta.env.DRY_RUN === "1" ?? process.env.DRY_RUN === "1" ?? true

main().catch(_ => {
  consola.error(_)
  process.exit(1)
})

async function main() {
  try {
    if (DRY_RUN) {
      return await Bun.$ /* sh */`bunx jsr publish --allow-dirty --allow-slow-types --dry-run`
    }

    const bumpPackage = await Bun.$ /* sh */`npm version prerelease --preid rc --no-git-tag-version`
    const version = bumpPackage.text().trim().replace(/^v/, "")

    // sync jsr.json version with package.json version
    await Bun.$ /* sh */`jq --arg version "${version}" '.version = $version' jsr.json > jsr.temp.json && mv jsr.temp.json jsr.json`

    return await Bun.$ /* sh */`bunx jsr publish --allow-dirty --allow-slow-types`
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    console.error(errorMessage)

    // revert changes
    await resetVersions()
    console.info("Reset package.json version")
  }
}

async function resetVersions() {
  const currentVersion = await Bun.$ /* sh */`jq --raw-output .version package.json`.text()
  const newVersion =
    await Bun.$ /* sh */`${currentVersion.trim()} | awk -F'[.-]' '{print $1"."$2"."$3"-"$4"."$6-1}'`
  await Bun.$ /* sh */`npm version ${newVersion} --no-git-tag-version`
}
