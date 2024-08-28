import { fallback, http, createPublicClient } from "viem"
import { createUnionClient } from "#mod.js"
import { sepolia } from "viem/chains"

const client = createPublicClient({
  chain: sepolia,
  transport: fallback([http("https://rpc.sepolia.org")])
})

createUnionClient({
  chainId: "11155111",
  transport: fallback([http("https://rpc.sepolia.org")])
}).approveTransaction

createUnionClient({
  chainId: "union-testnet-8",
  transport: http("https://rpc.testnet-8.union.build")
}) //.approveTransaction

// const account = getConnectorClient(config)

// const multiClient = createMultiUnionClient([
//   {
//     account,
//     chainId: "11155111",
//     transport: fallback([http("https://rpc.sepolia.org")])
//   },
//   {
//     chainId: 'union-testnet-8',
//     transport: fallback([http("https://rpc.testnet-8.union.build")])
//   }
// ])
