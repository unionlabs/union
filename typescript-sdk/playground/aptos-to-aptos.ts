#!/usr/bin/env bun
import "#patch.ts"
import {
  http,
  createUnionClient,
  hexStringToUint8Array,
  type TransferAssetsParameters
} from "#mod.ts"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"

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
  privateKey: new Ed25519PrivateKey(hexStringToUint8Array(PRIVATE_KEY))
})
console.info(aptosAccount.accountAddress.toString())

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
} satisfies TransferAssetsParameters<"2">

const simulateResult = await client.simulateTransaction(transferPayload)

if (simulateResult.isErr()) {
  consola.error("simulateResult error:", simulateResult.error)
  process.exit(1)
}

consola.success("simulateResult result:", simulateResult.value)

const transferResultSameChain = await client.transferAsset(transferPayload)

if (transferResultSameChain.isErr()) {
  consola.error("transferResultSameChain error:", transferResultSameChain.error)
  process.exit(1)
}

consola.success("transferResultSameChain result:", transferResultSameChain.value)
