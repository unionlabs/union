#!/usr/bin/env bun
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { UnionClient } from "#mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import { http, createWalletClient, fallback, getAddress } from "viem"
import contracts from "~root/versions/contracts.json" with { type: "json" }

/* `bun scripts/sepolia-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

console.info({ address: evmAccount.address })

const evmSigner = createWalletClient({
  chain: sepolia,
  account: evmAccount,
  transport: fallback([
    http(`https://rpc2.sepolia.org`),
    http(
      `https://special-summer-film.ethereum-sepolia.quiknode.pro/3e6a917b56620f854de771c23f8f7a8ed973cf7e/`
    ),
    http(`https://eth-sepolia.g.alchemy.com/v2/SQAcneXzJzITjplR7cwQhFUqF-SU-ds4`)
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

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789" // LINK contract address
const wOSMO_CONTRACT_ADDRESS = "0x3C148Ec863404e48d88757E88e456963A14238ef"
const USDC_CONTRACT_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

const currentContracts = contracts.find(c => c.latest === true) as (typeof contracts)[0]
const relayContractAddress = getAddress(currentContracts?.sepolia.UCS01)

const approve = await unionClient.approveEvmAssetTransfer({
  account: evmAccount,
  amount: 10n,
  denomAddress: LINK_CONTRACT_ADDRESS,
  relayContractAddress
})

console.log(approve)

const linkFromSepoliaToUnion = await unionClient.transferEvmAsset({
  account: evmAccount,
  receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
  denomAddress: LINK_CONTRACT_ADDRESS,
  amount: 2n,
  sourceChannel: "channel-13",
  simulate: true,
  relayContractAddress
})

console.log(linkFromSepoliaToUnion)
