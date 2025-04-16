#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { hexToBytes } from "../src/convert.ts"
import { createUnionClient } from "../src/mod.ts"
import { raise } from "../src/utilities/index.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/union-to-union.ts --private-key "..."` --estimate-gas */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) raise("Private key not found")
const ONLY_ESTIMATE_GAS = values["estimate-gas"] ?? false

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexToBytes(PRIVATE_KEY)),
  "union"
)

const [account] = await cosmosAccount.getAccounts()
if (!account) raise("no account found")

const client = createUnionClient({
  account: cosmosAccount,
  chainId: "union-testnet-9",
  transport: http("https://rpc.testnet-9.union.build")
})

const transaction = await client.cw20IncreaseAllowance({
  account: cosmosAccount,
  amount: 50n,
  gasPrice: { amount: "0.0025", denom: "muno" },
  // CW20 Minter
  spender: "union16zul4t9a9lx5g900c2mrsdfxsnheg6gazsefsgpxhaektqnsxe4s4cl6ek",
  contractAddress: "union1vu9he2ldfl6uf4wh8h4llmt8n8pdtlgrvmkpdmqtzew4culkuv9sxxke4q"
})

if (transaction.isErr()) {
  console.info("transaction failed")
  consola.error(transaction.error)
  process.exit(1)
}

consola.info(transaction.value)
process.exit(0)
