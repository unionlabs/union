#!/usr/bin/env bun
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { UnionClient } from "#v0/mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import contracts from "~root/versions/contracts.json" with { type: "json" }
import { http, erc20Abi, publicActions, createWalletClient, getAddress } from "viem"

/* `bun playground/to-sepolia.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const evmSigner = createWalletClient({
  chain: sepolia,
  account: evmAccount,
  transport: http(`https://eth-sepolia.g.alchemy.com/v2/SQAcneXzJzITjplR7cwQhFUqF-SU-ds4`)
}).extend(publicActions)

const unionClient = await UnionClient.connectWithSecret({
  rpcUrl: "https://rpc.testnet.bonlulu.uno",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  secretType: "key",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" },
  evmSigner
})

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789"
const wOSMO_CONTRACT_ADDRESS = "0x3C148Ec863404e48d88757E88e456963A14238ef"
const USDC_CONTRACT_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

const currentContracts = contracts.find(c => c.latest === true) as (typeof contracts)[0]
const relayContractAddress = getAddress(currentContracts?.sepolia.UCS01)

const approve = await evmSigner.writeContract({
  account: evmAccount,
  address: LINK_CONTRACT_ADDRESS,
  abi: erc20Abi,
  functionName: "approve",
  chain: sepolia,
  args: [
    "0xd0081080ae8493cf7340458eaf4412030df5feeb", // spender - SEPOLIA_UCS01_ADDRESS
    10n // amount
  ]
})

console.log(approve)

/**
 * 
cast send \ 
    --rpc-url https://rpc2.sepolia.org \
    --private-key 0x1bdd5c2105f62c51d72c90d9e5ca6854a94337bcbcbb0b959846b85813d69380 \
    0xd0081080ae8493cf7340458eaf4412030df5feeb \
    "send(string, string, bytes, (address, uint128)[], uint64, uint64)" "0x3d0eb16ad2619666dbde1921282cd885b58eeefe" "channel-0" "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed" "[(0x779877A7B0D9E8603169DdbD7836e478b4624789, 1)]" "8" "10000000"

 */

const osmoFromSepoliaToUnion = await unionClient.transferEvmAsset({
  account: evmAccount,
  receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
  denomAddress: LINK_CONTRACT_ADDRESS,
  amount: 1n,
  sourceChannel: "channel-8",
  relayContractAddress,
  simulate: true
})
console.log(osmoFromSepoliaToUnion)
