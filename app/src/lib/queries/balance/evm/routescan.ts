import * as v from "valibot"
import type { Address } from "viem"
import { raise } from "$lib/utilities"

const routescanTokenBalancesSchema = v.object({
  items: v.array(
    v.object({
      tokenAddress: v.pipe(v.string(), v.length(42)),
      tokenSymbol: v.string(),
      tokenQuantity: v.string()
    })
  )
})

export type EvmBalances = v.InferOutput<typeof routescanTokenBalancesSchema>

export async function getBalancesFromRoutescan({
  url,
  walletAddress
}: { url: string; walletAddress: string }) {
  let json: unknown

  try {
    const routescanUrl = `https://${url}/address/${walletAddress}/erc20-holdings`
    const response = await fetch(routescanUrl)
    if (!response.ok) raise("error fetching from routescan: non-200 status")
    json = await response.json()
  } catch (error) {
    if (error instanceof Error) {
      raise(`error fetching from routescan: ${error.message}`)
    }
    raise(`unknown error while fetching from routescan: ${JSON.stringify(error)}`)
  }
  const result = v.safeParse(routescanTokenBalancesSchema, json)

  if (!result.success) raise(`error parsing result ${JSON.stringify(result.issues)}`)

  return result.output.items.map(({ tokenAddress, tokenQuantity, tokenSymbol }) => ({
    balance: BigInt(tokenQuantity),
    gasToken: false,
    address: tokenAddress as Address,
    symbol: tokenSymbol
  }))
}
