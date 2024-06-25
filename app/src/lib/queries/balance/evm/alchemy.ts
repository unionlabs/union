import * as v from "valibot"
import { raise } from "$lib/utilities"

const alchemyTokenBalancesSchema = v.object({
  jsonrpc: v.string(),
  id: v.number(),
  result: v.object({
    address: v.pipe(v.string(), v.length(42)),
    tokenBalances: v.array(
      v.object({
        contractAddress: v.pipe(v.string(), v.length(42)),
        tokenBalance: v.string()
      })
    )
  })
})

export async function getBalancesFromAlchemy({
  url,
  walletAddress
}: { url: string; walletAddress: string }): Promise<
  Array<{
    name: string
    symbol: string
    address: string
    balance: string
    gasToken: boolean
  }>
> {
  let json: unknown

  try {
    const alchemyUrl = URL.canParse(url) ? url : `https://${url}`
    const response = await fetch(alchemyUrl, {
      method: "POST",
      body: JSON.stringify({
        id: 1,
        jsonrpc: "2.0",
        method: "alchemy_getTokenBalances",
        params: [walletAddress, "erc20"]
      })
    })
    if (!response.ok) raise("error fetching from alchemy: non-200 status")
    json = await response.json()
  } catch (error) {
    if (error instanceof Error) {
      raise(`error fetching from alchemy: ${error.message}`)
    }
    raise(`unknown error while fetching from alchemy: ${JSON.stringify(error)}`)
  }
  const result = v.safeParse(alchemyTokenBalancesSchema, json)

  if (!result.success) raise(`error parsing result ${JSON.stringify(result.issues)}`)

  return result.output.result.tokenBalances.map(token => ({
    name: token.contractAddress,
    symbol: token.contractAddress,
    address: token.contractAddress,
    balance: token.tokenBalance,
    gasToken: false
  }))
}
