#!/usr/bin/env bun
import "#patch.ts"
import { parseArgs } from "node:util"
import { cosmosHttp } from "#transport.ts"
import { createUnionClient } from "#mod.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { timestamp } from "../scripts/logger.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/**
 *
 *
 * W  I  P
 *
 *
 */

/* `bun playground/union-to-osmosis.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const stamp = timestamp()
const relayContractAddress = "union1eumfw2ppz8cwl8xdh3upttzp5rdyms48kqhm30f8g9u4zwj0pprqg2vmu3"

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

const client = createUnionClient({
  // @ts-expect-error
  evm: {},
  cosmos: {
    account: cosmosAccount,
    gasPrice: { amount: "0.0025", denom: "muno" },
    transport: cosmosHttp("https://rpc.testnet.bonlulu.uno")
  }
})

try {
  const hash = await client.transferAsset({
    amount: 1n,
    network: "cosmos",
    relayContractAddress,
    denomAddress: "muno",
    sourceChannel: "channel-27",
    path: ["union-testnet-8", "osmo-test-5"],
    recipient: "osmo14qemq0vw6y3gc3u3e0aty2e764u4gs5l32ydm0"
  })
  console.info(hash)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
