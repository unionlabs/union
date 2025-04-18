import { FetchHttpClient, HttpClient } from "@effect/platform"
import { Record as R, Option as O, Effect, pipe, Schedule } from "effect"
import type { Chain } from "viem"
import type { NoSuchElementException } from "effect/Cause"

export const resolveSafeTx = (
  chain: Chain,
  hash: `0x${string}`
): Effect.Effect<string, NoSuchElementException, never> =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient

    const networkName = chain.name

    const endpoint = `https://safe-transaction-mainnet.safe.global`
    const url = `${endpoint}/api/v1/multisig-transactions/${hash}`

    const response = yield* client.get(url)
    const json = response.json

    const result = pipe(
      json,
      // extract response data
      // TODO: replace with schema
      Effect.flatMap(x => R.get(x as Record<string, string>, "transactionHash")),
      // assume provided hash is already real one (?)
      Effect.catchIf(
        e => e._tag === "ResponseError" && e.response.status === 404,
        () => Effect.succeed(hash)
      )
    )

    return yield* result
  }).pipe(
    Effect.retryOrElse(
      Schedule.addDelay(Schedule.recurs(10), () => "500 millis"),
      () => O.none()
    ),
    Effect.provide(FetchHttpClient.layer),
    Effect.scoped
  )

Effect.runPromiseExit(resolveSafeTx(undefined as unknown as Chain, "0xHASH"))
