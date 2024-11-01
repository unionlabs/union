#!/usr/bin/env bun
import "#patch.ts"
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

const aptosAccount = Account.fromPrivateKey({
  privateKey: new Ed25519PrivateKey(PRIVATE_KEY)
})

const client = createUnionClient({
  chainId: "2",
  account: aptosAccount,
  transport: http("https://api.testnet.aptoslabs.com/v1")
})

const transferPayload = {
  memo: "",
  amount: 1n,
  simulate: false,
  authAccess: "key",
  account: aptosAccount,
  destinationChainId: "union-testnet-8",
  receiver: "union17ttpfu2xsmfxu6shl756mmxyqu33l5ljs5j6md",
  denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c"
} satisfies TransferAssetsParameters<"2">

const simulateResult = await client.simulateTransaction(transferPayload)

if (simulateResult.isErr()) {
  consola.error("simulateResult error:", simulateResult.error)
  process.exit(1)
}

consola.success("simulateResult:", simulateResult.value)

const transferResult = await client.transferAsset(transferPayload)

if (transferResult.isErr()) {
  consola.error("transferResult error:", transferResult.error)
  process.exit(1)
}

consola.success("transferResult:", transferResult.value)
