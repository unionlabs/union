#!/usr/bin/env bun
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { UnionClient } from "#/mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import { walletActionsEip5792 } from "viem/experimental"
import { http, erc20Abi, publicActions, createWalletClient } from "viem"

/* `bun scripts/sepolia-to-union.ts --private-key "..."` */

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
})
  .extend(publicActions)
  .extend(walletActionsEip5792())

const unionClient = await UnionClient.connectWithSecret({
  rpcUrl: "https://rpc.testnet.bonlulu.uno",
  bech32Prefix: "union",
  chainId: "union-testnet-8",
  secretType: "key",
  privateKeyOrMnemonic: PRIVATE_KEY,
  gas: { amount: "0.0025", denom: "muno" },
  evmSigner
})

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789" // LINK contract address
const wOSMO_CONTRACT_ADDRESS = "0x3C148Ec863404e48d88757E88e456963A14238ef"

const USDC_CONTRACT_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

const approve = await evmSigner.writeContract({
  account: evmAccount,
  address: LINK_CONTRACT_ADDRESS,
  abi: erc20Abi,
  functionName: "approve",
  chain: sepolia,
  args: [
    "0x3d0eb16ad2619666dbde1921282cd885b58eeefe", // spender - SEPOLIA_UCS01_ADDRESS
    10n // amount
  ]
})

console.log(approve)

const linkFromSepoliaToUnion = await unionClient.transferEvmAsset({
  account: evmAccount,
  receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
  denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789",
  amount: 1n,
  sourceChannel: "channel-0",
  sourcePort: "0x3d0eb16ad2619666dbde1921282cd885b58eeefe",
  contractAddress: "0x3d0eb16ad2619666dbde1921282cd885b58eeefe", // SEPOLIA_UCS01_ADDRESS
  simulate: true
})
console.log(linkFromSepoliaToUnion)
