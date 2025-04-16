#!/usr/bin/env bun
import "scripts/patch"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "../src/utilities/index.ts"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { http, createUnionClient, type TransferAssetsParametersLegacy } from "../src/mod.ts"

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
  destinationChainId: "2",
  receiver: "0xe3579557fd55ed8fab0d1e211eb1c05d56d74650e7070b703925493c38fe2aed",
  denomAddress: "0x9935a6a334e070bcecf5b1abb1c842c123572e63e70f0539d79610c32954c06c"
} satisfies TransferAssetsParametersLegacy<"2">

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
