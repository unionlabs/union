import * as v from "valibot"
import { raise } from "$lib/utilities"
import type { TokenBalance } from "./index.ts"

const cosmosBalancesResponseSchema = v.object({
  balances: v.array(
    v.object({
      denom: v.string(),
      amount: v.string()
    })
  )
})

export async function getCosmosChainBalances({
  url,
  walletAddress
}: { url: string; walletAddress: string }): Promise<Array<TokenBalance>> {
  let json: undefined | unknown

  try {
    url = url.startsWith("https") ? url : `https://${url}`
    const response = await fetch(`${url}/cosmos/bank/v1beta1/balances/${walletAddress}`)
    if (!response.ok) raise("invalid response")

    json = await response.json()
  } catch (error) {
    if (error instanceof Error) {
      raise(`error fetching balances from /cosmos/bank: ${error.message}`)
    }
    raise(`unknown error while fetching from /cosmos/bank: ${JSON.stringify(error)}`)
  }

  const result = v.safeParse(cosmosBalancesResponseSchema, json)

  if (!result.success) raise(`error parsing result ${JSON.stringify(result.issues)}`)
  return result.output.balances.map(x => ({
    name: x.denom,
    symbol: x.denom,
    address: x.denom,
    balance: BigInt(x.amount),
    gasToken: false,
    decimals: 0
  }))
}
