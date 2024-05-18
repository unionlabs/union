#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { UnionClient } from "#/mod.ts"

/* `bun scripts/from-osmosis.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const unionClient = await UnionClient.connectWithSecret({
  secretType: "key",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" },
  rpcUrl: "https://rpc.testnet.bonlulu.uno"
})

const account = await unionClient.getAccount()
const cwClient = await unionClient.signingCosmWasmClient()
const sendUnoToUnionAddress = await cwClient.sendTokens(
  account.address,
  "union1wv7yh86t0s4lw5lp9wdn59c22us75thmy5233q",
  [{ denom: "muno", amount: "100" }],
  "auto",
  "pepepopo"
)

console.log(sendUnoToUnionAddress)
