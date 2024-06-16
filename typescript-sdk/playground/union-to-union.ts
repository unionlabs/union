#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { UnionClient } from "#v0/mod.ts"
import { createUnionClient } from '#mod.ts'

/* `bun playground/from-osmosis.ts --private-key "..."` */

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

const account = await unionClient.getCosmosSdkAccount()
const cwClient = await unionClient.signingCosmWasmClient()
const sendUnoToUnionAddress = await cwClient.sendTokens(
  account.address,
  "union1eumfw2ppz8cwl8xdh3upttzp5rdyms48kqhm30f8g9u4zwj0pprqg2vmu3",
  [{ denom: "muno", amount: "25000000" }],
  "auto",
  "memo"
)
// 0xa833b03d8ed1228c4791cbfab22b3ed57954429f
console.log(sendUnoToUnionAddress.transactionHash)

const client = createUnionClient({
  
})
