#!/usr/bin/env bun
import { consola } from "./logger.ts"
import { offchainQuery } from "../src/mod.ts"

const [, , chainId] = process.argv

if (chainId) {
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
  process.exit(0)
}

const data = await offchainQuery.chains({
  includeEndpoints: true,
  includeContracts: true
})

if (!data) {
  consola.error("Chain not found")
  process.exit(1)
}

consola.info(JSON.stringify(data, undefined, 2))
