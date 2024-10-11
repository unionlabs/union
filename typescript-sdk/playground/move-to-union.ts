#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"

/* node --import=tsx playground/move-to-union.ts --private-key "e3579557fd55ed8fab0d1e211eb1c05d56d74650e7070b703925493c38fe2aed" */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" } // User's private key
  }
})

const PRIVATE_KEY = values["private-key"]

if (!PRIVATE_KEY) raise("Private key not found")

// Convert the hex string private key to a Uint8Array
const privateKeyBytes = Uint8Array.from(Buffer.from(PRIVATE_KEY, "hex"))

const moveAccount = Account.fromPrivateKey({
  privateKey: new Ed25519PrivateKey(privateKeyBytes)
})

// Create the Move client
const client = createUnionClient({
  chainId: "2",
  account: moveAccount,
  transport: http("https://api.testnet.aptoslabs.com/v1")
})

// const transferResult = await client.transferAsset({
//   memo: "",
//   amount: 1n,
//   receiver: "1363462745291c711144011c1305e737dd74ace69a5576612745e29a2e4fa1b5",
//   denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c",
//   destinationChainId: "union-testnet-8"
// } satisfies TransferAssetsParameters<"2">)

// // consola.success("Move Client created successfully", client)

// if (transferResult.isErr()) {
//   consola.error("Error transferring asset:", transferResult.error)
//   process.exit(1)
// }

// consola.success("Transfer result:", transferResult)

// consola.info("Calling transferAsset for same chain...");

const transferResultSameChain = await client.transferAsset({
  memo: "",
  amount: 1n,
  receiver: "0x2fb6eaaff3f29cedbcc89129a01aa60b2e4712ffd264b255244168a1bddea9ec",
  denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c",
  destinationChainId: "2"
} satisfies TransferAssetsParameters<"2">)
consola.success("transferResultSameChain result:", transferResultSameChain)

// consola.info("Calling simulateTransaction...")
// const simulateResult = await client.simulateTransaction({
//   memo: "",
//   amount: 1555n,
//   receiver: "1363462745291c711144011c1305e737dd74ace69a5576612745e29a2e4fa1b5",
//   denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c",
//   destinationChainId: "union-testnet-8"
// } satisfies TransferAssetsParameters<"2">)

// consola.success("simulateResult result:", simulateResult)
