#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { fromHex } from "@cosmjs/encoding"
import { GasPrice } from "@cosmjs/stargate"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"

/* `bun scripts/to-osmosis.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const decryptedWallet = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(fromHex(PRIVATE_KEY)),
  "union"
)

const [account] = await decryptedWallet.getAccounts()
if (!account) throw new Error("Account not found")

const cosmwasmClient = await SigningCosmWasmClient.connectWithSigner(
  "https://rpc.testnet.bonlulu.uno",
  decryptedWallet,
  { gasPrice: GasPrice.fromString(`0.0025muno`) }
)

const contractAddress = "union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7"
const osmoFromUnionToOsmosis = await cosmwasmClient.executeMultiple(
  account.address,
  [
    {
      msg: {
        transfer: {
          channel: "channel-0",
          receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd".slice(2),
          memo: "sending wrapped OSMO from Union to Sepolia"
        }
      },
      contractAddress,
      funds: [
        {
          amount: "6",
          denom: `factory/union124t57vjgsyknnhmr3fpkmyvw2543448kpt2xhk5p5hxtmjjsrmzsjyc4n7/0xc5775fca1b3285dc8b749d58b227527211c108b8d3`
        }
      ]
    }
  ],
  "auto"
)

console.log(osmoFromUnionToOsmosis.transactionHash)
