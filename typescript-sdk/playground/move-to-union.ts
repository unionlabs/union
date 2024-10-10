#!/usr/bin/env bun

import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"

/* `bun playground/move-playground.ts --private-key "..." --chain-id "move-testnet-1" --rpc-url "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" }, // User's private key
    "chain-id": { type: "string" }, // Chain ID for the Move chain
    "rpc-url": { type: "string" } // Move RPC URL
  }
})

const PRIVATE_KEY = values["private-key"]
const CHAIN_ID = values["chain-id"]
const RPC_URL = values["rpc-url"]

if (!PRIVATE_KEY) raise("Private key not found")
if (!CHAIN_ID) raise("Chain ID not provided")
if (!RPC_URL) raise("RPC URL not provided")

// Convert the hex string private key to a Uint8Array
const privateKeyBytes = Uint8Array.from(Buffer.from(PRIVATE_KEY, "hex"))

const moveAccount = Account.fromPrivateKey({
  privateKey: new Ed25519PrivateKey(privateKeyBytes)
})

try {
  // Create the Move client
  const client = createUnionClient({
    account: `0x${PRIVATE_KEY}`,
    chainId: CHAIN_ID as "2", // Adjust according to your setup
    transport: http(RPC_URL)
  })

  const transferResult = await client.transferAsset({
    memo: "Test transfer",
    amount: 1000n, // Test amount in smallest units
    receiver: "f2e43321983ebfc75ff689e625b0cdccf56365560d2d970ea4ec0b898728b954",
    denomAddress: "f2e43321983ebfc75ff689e625b0cdccf56365560d2d970ea4ec0b898728b954",
    destinationChainId: "union-testnet-8"
  } satisfies TransferAssetsParameters<"2">)

  consola.success("Transfer result:", transferResult)
  consola.success("Move Client created successfully", client)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  consola.error("Error creating Move client:", errorMessage)
} finally {
  process.exit(0)
}
