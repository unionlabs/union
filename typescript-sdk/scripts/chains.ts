#!/usr/bin/env bun
import { consola } from "./logger.ts"
import { offchainQuery } from "#mod.ts"

const [, , chainId] = process.argv

if (!chainId) {
  consola.error("Chain ID is required")
  process.exit(1)
}

const data = await offchainQuery.chain({
  chainId,
  includeEndpoints: true,
  includeContracts: true
})

if (!data) {
  consola.error("Chain not found")
  process.exit(1)
}

consola.info(JSON.stringify(data, undefined, 2))
