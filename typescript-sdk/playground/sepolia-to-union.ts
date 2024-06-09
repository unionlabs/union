#!/usr/bin/env bun
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { UnionClient } from "#mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import { http, createWalletClient, fallback, getAddress } from "viem"
import contracts from "~root/versions/contracts.json" with { type: "json" }
import { createUnionClient } from "#client.ts"
/* `bun playground/sepolia-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

// console.info({ address: evmAccount.address })

const evmSigner = createWalletClient({
  chain: sepolia,
  account: evmAccount,
  transport: fallback([
    http(`https://rpc2.sepolia.org`)
    // http(
    //   `https://special-summer-film.ethereum-sepolia.quiknode.pro/3e6a917b56620f854de771c23f8f7a8ed973cf7e/`
    // ),
    // http(`https://eth-sepolia.g.alchemy.com/v2/SQAcneXzJzITjplR7cwQhFUqF-SU-ds4`)
  ])
})

const unionClient = await UnionClient.connectWithSecret({
  rpcUrl: "https://rpc.testnet.bonlulu.uno",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  secretType: "key",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" },
  evmSigner
})

const CHANNEL = "channel-22"

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789" // LINK contract address
const wOSMO_CONTRACT_ADDRESS = "0x3C148Ec863404e48d88757E88e456963A14238ef"
const USDC_CONTRACT_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

const currentContracts = contracts.find(c => c.latest === true) as (typeof contracts)[0]
const relayContractAddress = getAddress(currentContracts?.sepolia.UCS01)

// const approve = await unionClient.approveEvmAssetTransfer({
//   account: evmAccount,
//   amount: 10n,
//   denomAddress: USDC_CONTRACT_ADDRESS,
//   relayContractAddress
// })

// console.log({ approve })

// const usdcFromSepoliaToUnion = await unionClient.transferEvmAsset({
//   account: evmAccount,
//   receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
//   denomAddress: USDC_CONTRACT_ADDRESS,
//   amount: 1n,
//   sourceChannel: CHANNEL,
//   simulate: true,
//   relayContractAddress
// })

// console.log(usdcFromSepoliaToUnion)

const client = createUnionClient({
  evmAccount,
  evmRpcUrl: "https://rpc2.sepolia.org",
  cosmosRpcUrl: "https://rpc.testnet.bonlulu.uno"
})

const transfer = await client.transferAsset({
  sourceChainId: "11115511",
  destinationChainId: "union-testnet-8",
  evmAccount,
  receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
  sourceChannel: CHANNEL,
  amount: 1n,
  denomAddress: USDC_CONTRACT_ADDRESS,
  relayContractAddress,
  simulate: true
})

console.info(transfer)

/**
cast send \                                                                                                                                 hussein@nerded
    --rpc-url https://rpc2.sepolia.org \
    --private-key 0x227bfab7b601429981d3fbffb1b7625fb13464965cd4368ae48090c8d44b417e \
    0xd0081080ae8493cf7340458eaf4412030df5feeb \
    "send(string, bytes, (address, uint128)[], (uint64, uint64), uint64)" "channel-22" "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed" "[(0x779877A7B0D9E8603169DdbD7836e478b4624789, 1)]" "(100, 1000000000)" "0"

 */
