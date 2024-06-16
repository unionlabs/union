#!/usr/bin/env bun
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { UnionClient } from "#v0/mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import { http, publicActions, createWalletClient, fallback } from "viem"

/* `bun playground/sepolia-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY =
  values["private-key"] || "1bdd5c2105f62c51d72c90d9e5ca6854a94337bcbcbb0b959846b85813d69380"
if (!PRIVATE_KEY) throw new Error("Private key not found")

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const evmSigner = createWalletClient({
  chain: sepolia,
  account: evmAccount,
  // transport: http(`https://eth-sepolia.g.alchemy.com/v2/SQAcneXzJzITjplR7cwQhFUqF-SU-ds4`)
  // transport: http(`https://rpc2.sepolia.org`)
  transport: fallback([
    http(`https://rpc2.sepolia.org`)
    // http(
    //   `https://special-summer-film.ethereum-sepolia.quiknode.pro/3e6a917b56620f854de771c23f8f7a8ed973cf7e/`
    // )
  ])
}).extend(publicActions)

const unionClient = await UnionClient.connectWithSecret({
  rpcUrl: "https://union-testnet-rpc.polkachu.com",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  secretType: "key",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" },
  evmSigner
})

// const balances = await unionClient.getBalances()
// console.info(balances)
