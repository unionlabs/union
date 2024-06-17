#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { hexStringToUint8Array } from "#convert.ts"
import { createUnionClient, cosmosHttp } from "#mod.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/osmosis-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "osmo"
)

const unionClient = createUnionClient({
  // @ts-expect-error
  evm: {},
  cosmos: {
    account: cosmosAccount,
    gasPrice: { amount: "0.0025", denom: "uosmo" },
    transport: cosmosHttp("https://rpc.osmo.test.yieldpay.finance")
  }
})

try {
  const hash = await unionClient.transferAsset({
    amount: 1n,
    network: "cosmos",
    denomAddress: "uosmo",
    sourcePort: "transfer",
    sourceChannel: "channel-8075",
    path: ["osmo-test-5", "union-testnet-8"],
    // gasPrice: { amount: "0.0025", denom: "uosmo" },
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
  })

  console.info(hash)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
