#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { http, createUnionClient, type TransferAssetsParameters } from "#mod.ts"

/* node --import=tsx playground/aptos-to-union.ts --private-key $PRIVATE_KEY */

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

const aptosAccount = Account.fromPrivateKey({
  privateKey: new Ed25519PrivateKey(privateKeyBytes)
})

// Create the Aptos client
const client = createUnionClient({
  chainId: "2",
  account: aptosAccount,
  transport: http("https://api.testnet.aptoslabs.com/v1")
})

const transferPayload = {
  memo: "",
  amount: 1n,
  receiver: "1363462745291c711144011c1305e737dd74ace69a5576612745e29a2e4fa1b5",
  denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c",
  destinationChainId: "union-testnet-8"
} satisfies TransferAssetsParameters<"2">

const simulateResult = await client.simulateTransaction(transferPayload)

if (simulateResult.isErr()) {
  consola.error("simulateResult error:", simulateResult.error)
  process.exit(1)
}

consola.success("simulateResult result:", simulateResult.value)

const transferResult = await client.transferAsset(transferPayload)

if (transferResult.isErr()) {
  consola.error("transferResult error:", transferResult.error)
  process.exit(1)
}

consola.success("transferResult result:", transferResult.value)
