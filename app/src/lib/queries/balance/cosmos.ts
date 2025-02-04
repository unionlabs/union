import * as v from "valibot"
import { err, errAsync, ok, ResultAsync } from "neverthrow"
import type { RawBalances } from "."
import { toHex } from "viem"
import { isValidBech32Address } from "@unionlabs/client"

const cosmosBalancesResponseSchema = v.object({
  balances: v.array(
    v.object({
      denom: v.string(),
      amount: v.string()
    })
  )
})

const fetchJson = (url: string) => {
  return ResultAsync.fromPromise(
    fetch(url).then(response => {
      if (!response.ok) {
        throw new Error(`HTTP error for url ${url} status: ${response.status}`)
      }
      return response.json()
    }),
    e =>
      new Error(`Failed to fetch data from url ${url} with error: ${(e as Error).message}`, {
        cause: e
      })
  )
}

export function getCosmosChainBalances({
  url,
  walletAddress
}: { url: string; walletAddress: string }): ResultAsync<RawBalances, Error> {
  url = url.startsWith("https") ? url : `https://${url}`
  if (!isValidBech32Address(walletAddress))
    return errAsync(new Error(`invalid cosmos wallet address provided: ${walletAddress}`))
  return fetchJson(`${url}/cosmos/bank/v1beta1/balances/${walletAddress}`)
    .andThen(json => {
      const result = v.safeParse(cosmosBalancesResponseSchema, json)
      return result.success
        ? ok(result.output)
        : err(new Error("Validation failed:", { cause: result.issues }))
    })
    .map(balances =>
      balances.balances.reduce((acc, cur) => {
        acc[toHex(cur.denom)] = cur.amount
        return acc
      }, {})
    )
}
