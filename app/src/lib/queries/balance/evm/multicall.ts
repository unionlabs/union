import { getPublicClient } from "@wagmi/core"
import { raise } from "$lib/utilities/index.ts"
import { config } from "$lib/wallet/evm/config.ts"
import { erc20Abi, getAddress, type Address } from "viem"
import type { NoRepetition } from "$lib/utilities/types.ts"

export async function erc20ReadMulticall({
  address,
  chainId,
  contractAddresses,
  multicallOptions = {
    functionNames: ["balanceOf"]
  }
}: {
  address: Address
  chainId: string
  contractAddresses: Array<Address>
  multicallOptions?: {
    functionNames: NoRepetition<"name" | "balanceOf" | "decimals" | "symbol">
  }
}) {
  const validAddress = getAddress(address)
  const chain = config.chains.find(chain => String(chain.id) === chainId)
  if (!chain) raise(`chain with id ${chainId} not found`)

  const client = getPublicClient(config, { chainId: chain.id })

  const contracts = contractAddresses.flatMap(contractAddress =>
    multicallOptions.functionNames.map(functionName => ({
      functionName,
      abi: erc20Abi,
      address: contractAddress,
      args: functionName === "balanceOf" ? [validAddress] : undefined
    }))
  )

  const results = await client.multicall({ contracts })

  return results.reduce(
    (accumulator, { result }, index) => {
      if (index % multicallOptions.functionNames.length === 0) accumulator.push({})

      const currentResult = accumulator[accumulator.length - 1]
      const fn = multicallOptions.functionNames[index % multicallOptions.functionNames.length]
      currentResult[fn === "balanceOf" ? "balance" : fn] = result ?? (fn === "decimals" ? 0 : "")
      return accumulator
    },
    [] as Array<Record<string, any>>
  )
}
