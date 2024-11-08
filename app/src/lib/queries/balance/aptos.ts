import * as v from "valibot"
import { raise } from "$lib/utilities"

/**
 * v2: newer version of the token standard which we use
 */
const getFungibleAssetBalancesQuery = /* GraphQL */ `
  query GetFungibleAssetBalances(
    $address: String, 
    $token_standard: String = "v2"
  ) {
    data: current_fungible_asset_balances(
      where: {
        owner_address: { _eq: $address },
        token_standard: { _eq: $token_standard }
      }
      order_by: { amount: desc }
    ) {
      amount
      asset_type
      metadata {
        name
        symbol
        decimals
        supply_v2
      }
    }
  }
`

const aptosBalancesResponseSchema = v.object({
  data: v.object({
    data: v.array(
      v.object({
        amount: v.number(),
        asset_type: v.string(),
        metadata: v.object({
          name: v.string(),
          symbol: v.string(),
          decimals: v.number(),
          supply_v2: v.number()
        })
      })
    )
  })
})

export async function getAptosChainBalances({
  url,
  walletAddress
}: { url: string; walletAddress: string }) {
  url = url.startsWith("https") ? url : `https://${url}`
  const response = await fetch(`${url}/graphql`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: getFungibleAssetBalancesQuery,
      variables: {
        token_standard: "v2",
        address: walletAddress
      }
    })
  })

  if (!response.ok) raise("invalid response")
  const json = await response.json()

  const result = v.safeParse(aptosBalancesResponseSchema, json)

  if (!result.success) raise(`error parsing result ${JSON.stringify(result.issues)}`)
  return result.output.data.data.map(x => ({
    address: x.asset_type,
    symbol: x.metadata.symbol,
    balance: BigInt(x.amount),
    decimals: x.metadata.decimals
  }))
}
