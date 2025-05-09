// query.ts
import axios from "axios"
import { Data, Effect, pipe } from "effect"
import { extractErrorDetails } from "../utils/extract-error-details.js"
import { QueryContractError } from "./contract.js"

/**
 * Error type for HttpRequest execution failures
 */
export class HttpRequestFailed extends Data.TaggedError("HttpRequestFailed")<{
  status: number
  body: unknown
}> {}

/**
 * Query a contract at an explicit block height.
 */
export function queryContractSmartAtHeight<T = unknown>(
  restEndpoint: string,
  contractAddress: string,
  queryMsg: Record<string, unknown>,
  height: number,
) {
  const base = restEndpoint.replace(/\/+$/, "")
  const encoded = btoa(JSON.stringify(queryMsg))
  const url = `${base}/cosmwasm/wasm/v1/contract/${contractAddress}/smart/${encoded}`

  return pipe(
    Effect.tryPromise({
      try: async () => {
        const resp = await axios.get(url, {
          headers: {
            "Content-Type": "application/json",
            "x-cosmos-block-height": height.toString(),
          },
        })
        if (resp.status < 200 || resp.status >= 300) {
          throw new HttpRequestFailed({ status: resp.status, body: resp.data })
        }
        return resp.data as T
      },
      catch: (err: unknown) =>
        err instanceof HttpRequestFailed
          ? err
          : new QueryContractError({ cause: extractErrorDetails(err as Error) }),
    }),
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
  )
}

/**
 * Fetch an account's balance for a denom at a specific block height.
 */
export function getBalanceAtHeight(
  restEndpoint: string,
  address: string,
  denom: string,
  height: number,
) {
  const base = restEndpoint.replace(/\/+$/, "")
  const url = `${base}/cosmos/bank/v1beta1/balances/${address}`

  return pipe(
    Effect.tryPromise({
      try: async () => {
        const resp = await axios.get(url, {
          headers: {
            "Content-Type": "application/json",
            "x-cosmos-block-height": height.toString(),
          },
        })
        if (resp.status < 200 || resp.status >= 300) {
          throw new HttpRequestFailed({ status: resp.status, body: resp.data })
        }

        const data: any = resp.data
        const entry = data.balances.find((b: { denom: string }) => b.denom === denom)
        return entry ? BigInt(entry.amount) : null
      },
      catch: (err: unknown) =>
        err instanceof HttpRequestFailed
          ? err
          : new HttpRequestFailed({
            status: 0,
            body: extractErrorDetails(err as Error),
          }),
    }),
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
  )
}
