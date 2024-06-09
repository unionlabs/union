#!/usr/bin/env bun
import "#patch.ts"
import { parseArgs } from "node:util"
import { UnionClient } from "#mod.ts"
import { raise } from "#utilities.ts"
import { GasPrice } from "@cosmjs/stargate"
import { hexStringToUint8Array } from "#convert.ts"
import { privateKeyToAccount } from "viem/accounts"
import { consola, timestamp } from "../scripts/logger.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import contracts from "~root/versions/contracts.json" with { type: "json" }
import { createUnionClient } from "#client.ts"

/* `bun playground/union-to-sepolia.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "tx-count": { type: "string", default: "1" }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")
const TX_COUNT = Number(values["tx-count"])

const CHANNEL = "channel-28"

const ucs01Contract =
  contracts.find(c => c.latest === true)?.union.UCS01 ?? raise("UCS01 contract not found")

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

consola.box(`Sending ${TX_COUNT} transactions from Union to Sepolia`)

const unionClient = await UnionClient.connectWithSecret({
  rpcUrl: "https://rpc.testnet.bonlulu.uno",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  secretType: "key",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" }
})

const { address } = await unionClient.getCosmosSdkAccount()

const contractAddress = "union1eumfw2ppz8cwl8xdh3upttzp5rdyms48kqhm30f8g9u4zwj0pprqg2vmu3"
const stamp = timestamp()
// const unionToSepoliaTransactions: Array<ExecuteInstruction> = Array.from(
//   { length: TX_COUNT },
//   (_, index) => ({
//     contractAddress,
//     msg: {
//       transfer: {
//         channel: "channel-28",
//         receiver: evmAccount.address.slice(2),
//         memo: `${index} - ${stamp} Sending UNO from Union to ${evmAccount.address} on Sepolia`
//       }
//     },
//     funds: [{ amount: (index + 1).toString(), denom: `muno` }]
//   })
// )

// const transactionResults = await Array.fromAsync(
//   unionToSepoliaTransactions,
//   async transaction =>
//     unionClient.transferAssets({ kind: "cosmwasm", instructions: [transaction] }),
//   { concurrency: 1 }
// )
// console.info(stamp)
// consola.info(
//   JSON.stringify(
//     transactionResults.map(item => item.transactionHash),
//     undefined,
//     2
//   )
// )

const cosmosSigner = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

// const transfer = await cosmwasmTransfer({
//   cosmosSigner: await DirectSecp256k1Wallet.fromKey(
//     Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
//     "union"
//   ),
//   cosmosRpcUrl: "https://rpc.testnet.bonlulu.uno",
//   gasPrice: GasPrice.fromString("0.0025muno"),
//   instructions: [
//     {
//       contractAddress,
//       msg: {
//         transfer: {
//           channel: CHANNEL,
//           receiver: evmAccount.address.slice(2),
//           memo: `${stamp} Sending UNO from Union to ${evmAccount.address} on Sepolia`
//         }
//       },
//       funds: [
//         {
//           amount: "1",
//           denom:
//             "factory/union1eumfw2ppz8cwl8xdh3upttzp5rdyms48kqhm30f8g9u4zwj0pprqg2vmu3/0xbf41fec2bba5519a54171fc02966728e29e3d18adc"
//         }
//       ]
//     }
//   ]
// })

// console.info(transfer.transactionHash)

const client = createUnionClient({
  cosmosRpcUrl: "https://rpc.testnet.bonlulu.uno",
  evmRpcUrl: ""
})

const hash = await client.transferAsset({
  relayContractAddress: ucs01Contract,
  path: ["union-testnet-8", "11155111"],
  amount: 1n,
  denomAddress: "muno",
  receiver: evmAccount.address,
  cosmosSigner,
  sourceChannel: CHANNEL,
  network: "cosmos",
  gasPrice: GasPrice.fromString("0.0025muno"),
  cosmosRpcUrl: "https://rpc.testnet.bonlulu.uno"
})

console.info(hash)
