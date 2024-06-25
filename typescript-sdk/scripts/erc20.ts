import * as chains from "viem/chains"
import { parseArgs } from "node:util"
import { raise } from "#utilities/index.ts"
import { createPublicClient, erc20Abi, fallback, getAddress, http } from "viem"

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "chain-id": { type: "string" },
    address: { type: "string" }
  }
})

const chainId = values["chain-id"] ?? raise("Chain ID is required")
const address = getAddress(values["address"] ?? raise("Address is required"))
const chain = Object.values(chains).find(chain => chain.id === Number(chainId))

const client = createPublicClient({
  transport: fallback([http(chain?.rpcUrls.default.http.at(0))])
})

const [name, symbol, decimals] = await Promise.all(
  [
    {
      abi: erc20Abi,
      address,
      functionName: "name"
    } as const,
    {
      abi: erc20Abi,
      address,
      functionName: "symbol"
    } as const,
    {
      abi: erc20Abi,
      address,
      functionName: "decimals"
    } as const
  ].map(contract => client.readContract(contract))
)

console.info({ address, name, symbol, decimals })
