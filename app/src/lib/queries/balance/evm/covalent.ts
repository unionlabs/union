import * as v from "valibot"
import { raise } from "$lib/utilities"
import { KEY } from "$lib/constants/keys"
import type { Balance } from "../types.ts"

const covalentTokenBalancesSchema = v.object({
  data: v.object({
    address: v.string(),
    updated_at: v.string(),
    next_update_at: v.string(),
    quote_currency: v.string(),
    chain_id: v.number(),
    chain_name: v.string(),
    items: v.array(
      v.object({
        contract_decimals: v.union([v.number(), v.null()]),
        contract_name: v.union([v.string(), v.null()]),
        contract_ticker_symbol: v.union([v.string(), v.null()]),
        contract_address: v.string(),
        supports_erc: v.union([v.array(v.string()), v.null()]),
        logo_url: v.union([v.string(), v.null()]),
        contract_display_name: v.union([v.string(), v.null()]),
        logo_urls: v.object({
          token_logo_url: v.union([v.string(), v.null()]),
          protocol_logo_url: v.union([v.string(), v.null()]),
          chain_logo_url: v.union([v.string(), v.null()])
        }),
        last_transferred_at: v.string(),
        native_token: v.boolean(),
        type: v.string(),
        is_spam: v.boolean(),
        balance: v.string(),
        balance_24h: v.string(),
        quote_rate: v.union([v.string(), v.null()]),
        quote_rate_24h: v.union([v.string(), v.null()]),
        quote: v.union([v.string(), v.null()]),
        pretty_quote: v.union([v.string(), v.null()]),
        quote_24h: v.union([v.string(), v.null()]),
        pretty_quote_24h: v.union([v.string(), v.null()]),
        protocol_metadata: v.union([v.string(), v.null()]),
        nft_data: v.union([v.string(), v.null()])
      })
    )
  }),
  error: v.boolean(),
  error_message: v.union([v.string(), v.null()]),
  error_code: v.union([v.number(), v.null()])
})

// https://www.covalenthq.com/docs/networks/
export async function getBalancesFromCovalent({
  url,
  address
}: { address: string; url: string }): Promise<Array<Balance>> {
  url ||= URL.canParse(url) ? url : `https://${url}`
  const searchParams = new URLSearchParams({
    nft: "false",
    "no-spam": "true",
    "no-nft-asset-metadata": "false"
  })
  const response = await fetch(`${url}${address}/balances_v2/?${searchParams.toString()}`, {
    method: "GET",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
      Authorization: `Basic ${KEY.RPC.COVALENT}`
    }
  })

  const json = await response.json()
  const result = v.safeParse(covalentTokenBalancesSchema, json)
  if (!result.success) raise(`error parsing result ${JSON.stringify(result.issues)}`)

  return result.output.data.items.map(token => ({
    name: token.contract_name ?? token.contract_address,
    symbol: token.contract_ticker_symbol ?? token.contract_address,
    address: token.contract_address,
    balance: BigInt(token.balance),
    gasToken: token.native_token
  }))
}
