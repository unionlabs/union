import { getPublicClient } from "@wagmi/core"
import { raise } from "$lib/utilities/index.ts"
import { config } from "$lib/wallet/evm/config.ts"
import { erc20Abi, getAddress, type Address } from "viem"
import type { NoRepetition } from "$lib/utilities/types.ts"

/**
 * @example
 * ```ts
 * const results = await erc20ReadMulticall({
 *   chainId: "11155111",
 *   address: "0xf4212614C7Fe0B3feef75057E88b2E77a7E23e83",
 *   functionNames: ["balanceOf"], // only call balanceOf for each contract
 *   contractAddresses: [
 *     "0x0021c1948b470167430410c04236a969c34659f8",
 *     "0x013cb2854daad8203c6686682f5d876e5d3de4a2",
 *     "0x015225038e0571c04c1a0c8d981876d6b40ab4bc",
 *   ]
 * })
 * ```
 */
export async function erc20ReadMulticall({
  address,
  chainId,
  contractAddresses,
  functionNames = ["balanceOf"]
}: {
  address: Address
  chainId: string
  contractAddresses: Array<Address>
  functionNames: NoRepetition<"name" | "balanceOf" | "decimals" | "symbol">
}) {
  const validAddress = getAddress(address)
  const chain = config.chains.find(chain => String(chain.id) === chainId)
  if (!chain) raise(`chain with id ${chainId} not found`)

  const client = getPublicClient(config, { chainId: chain.id })

  const contracts = contractAddresses.flatMap(contractAddress =>
    functionNames.map(functionName => ({
      functionName,
      abi: erc20Abi,
      address: contractAddress,
      args: functionName === "balanceOf" ? [validAddress] : undefined
    }))
  )

  const results = await client.multicall({ contracts })

  return results.reduce(
    (accumulator, { result }, index) => {
      if (index % functionNames.length === 0) accumulator.push({})

      const currentResult = accumulator[accumulator.length - 1]
      const fn = functionNames[index % functionNames.length]
      currentResult[fn === "balanceOf" ? "balance" : fn] = result ?? (fn === "decimals" ? 0 : "")
      return accumulator
    },
    [] as Array<Record<string, any>>
  )
}
