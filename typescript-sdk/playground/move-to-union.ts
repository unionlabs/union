#!/usr/bin/env bun

import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"

/*  npx tsx playground/move-to-union.ts --private-key "bd84d9e61094ce511c72d5942ae59af556023e3fce4fd74ab5ef5167c4314e5b" --chain-id "2" */

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
let RPC_URL = values["rpc-url"]

if (!PRIVATE_KEY) raise("Private key not found")

if (!CHAIN_ID) raise("Chain ID not provided")

if (!RPC_URL) RPC_URL = "https://api.testnet.aptoslabs.com/v1"

// Convert the hex string private key to a Uint8Array
const privateKeyBytes = Uint8Array.from(Buffer.from(PRIVATE_KEY, "hex"))

const moveAccount = Account.fromPrivateKey({
  privateKey: new Ed25519PrivateKey(privateKeyBytes)
})

let client = null
try {
  // Create the Move client
  client = createUnionClient({
    account: moveAccount,
    chainId: CHAIN_ID as "2", // Adjust according to your setup
    transport: http(RPC_URL)
  })
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  consola.error("Error creating Move client:", errorMessage)
  process.exit(0)
}

consola.info("Calling simulateTransaction...")
const simulateResult = await client.simulateTransaction({
  memo: "",
  amount: 1555n,
  receiver: "1363462745291c711144011c1305e737dd74ace69a5576612745e29a2e4fa1b5",
  denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c",
  destinationChainId: "union-testnet-8"
} satisfies TransferAssetsParameters<"2">)

consola.success("simulateResult result:", simulateResult)
// consola.info("\n\n\n\n---------------------------------------------------\n\n\n\n")

// consola.info("Calling transferAsset...");
// const transferResult = await client.transferAsset({
//   memo: "",
//   amount: 1n,
//   receiver: "1363462745291c711144011c1305e737dd74ace69a5576612745e29a2e4fa1b5",
//   denomAddress:
//     "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c",
//   destinationChainId: "union-testnet-8",
// } satisfies TransferAssetsParameters<"2">);

// consola.success("Transfer result:", transferResult);
// consola.info("\n\n\n\n---------------------------------------------------\n\n\n\n");

// consola.info("Calling transferAsset for same chain...");
// const transferResultSameChain = await client.transferAsset({
//   memo: "",
//   amount: 1n,
//   receiver: "0xe3579557fd55ed8fab0d1e211eb1c05d56d74650e7070b703925493c38fe2aed",
//   denomAddress:
//     "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c",
//   destinationChainId: "2",
// } satisfies TransferAssetsParameters<"2">);
// consola.success("transferResultSameChain result:", transferResultSameChain);
