import { HttpClient, HttpClientRequest } from "@effect/platform"
import { Data, Effect, pipe } from "effect"
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
    Effect.gen(function*() {
      const request = HttpClientRequest.get(url).pipe(
        HttpClientRequest.setHeaders({
          "Content-Type": "application/json",
          "x-cosmos-block-height": height.toString(),
        }),
      )

      const client = yield* HttpClient.HttpClient
      const response = yield* client.execute(request)
      const data = yield* response.json

      return data as T
    }),
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
    Effect.catchAll((err) =>
      err instanceof HttpRequestFailed
        ? Effect.fail(err)
        : Effect.fail(new QueryContractError({ cause: err }))
    ),
  )
}

// /**
//  * Fetch an account's balance for a denom at a specific block height.
//  */
export function getBalanceAtHeight(
  restEndpoint: string,
  address: string,
  denom: string,
  height: number,
) {
  const base = restEndpoint.replace(/\/+$/, "")
  const url = `${base}/cosmos/bank/v1beta1/balances/${address}`
  return pipe(
    Effect.gen(function*() {
      const request = HttpClientRequest.get(url).pipe(
        HttpClientRequest.setHeaders({
          "Content-Type": "application/json",
          "x-cosmos-block-height": height.toString(),
        }),
      )

      const client = yield* HttpClient.HttpClient
      const response = yield* client.execute(request)
      const raw = yield* response.json

      const data = raw as {
        balances: Array<{ denom: string; amount: string }>
      }

      const entry = data.balances.find((b) => b.denom === denom)
      return entry ? BigInt(entry.amount) : null
    }),
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
    Effect.catchAll((err) =>
      err instanceof HttpRequestFailed
        ? Effect.fail(err)
        : Effect.fail(new QueryContractError({ cause: err }))
    ),
  )
}
