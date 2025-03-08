import { Data, Effect, Option, Schema, Schedule } from "effect"
import type { TimeoutException } from "effect/Cause"
import { fetchDecode } from "$lib/utils/queries"
import type { DurationInput } from "effect/Duration"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { Chain } from "$lib/schema/chain"
import type { AddressCosmosCanonical, AddressCosmosDisplay } from "$lib/schema/address"
import { FetchHttpClient } from "@effect/platform"

export type FetchCosmosBalanceError = TimeoutException | QueryBankBalanceError | CreateClientError

export class QueryBankBalanceError extends Data.TaggedError("QueryBankBalanceError")<{
  cause: unknown
}> {}

export class CreateClientError extends Data.TaggedError("CreateClientError")<{
  cause: unknown
}> {}

// Schema for the balance response from Cosmos chain
export const CosmosBalanceSchema = Schema.Struct({
  balance: Schema.Struct({
    amount: Schema.String,
    denom: Schema.String
  })
})

const fetchCosmosBalance = ({
  rpcUrl,
  walletAddress,
  denom
}: {
  rpcUrl: string
  walletAddress: AddressCosmosDisplay
  denom: TokenRawDenom
}) =>
  fetchDecode(
    CosmosBalanceSchema,
    `${rpcUrl}/cosmos/bank/v1beta1/balances/${walletAddress}/by_denom?denom=${denom}`
  ).pipe(
    Effect.map(response => response.balance.amount),
    Effect.mapError(err => new QueryBankBalanceError({ cause: err }))
  )

export const createCosmosBalanceQuery = ({
  chain,
  tokenAddress,
  walletAddress,
  refetchInterval,
  writeData,
  writeError
}: {
  chain: Chain
  tokenAddress: TokenRawDenom
  walletAddress: AddressCosmosCanonical
  refetchInterval: DurationInput
  writeData: (data: RawTokenBalance) => void
  writeError: (error: Option.Option<FetchCosmosBalanceError>) => void
}) => {
  const fetcherPipeline = Effect.gen(function* (_) {
    yield* Effect.log(`starting cosmos balances fetcher for ${walletAddress}:${tokenAddress}`)

    // TODO: Get RPC URL from chain config
    const rpcUrl = "https://rest.testnet-9.union.build"
    const displayAddress = yield* chain.toCosmosDisplay(walletAddress)
    const balance = yield* Effect.retry(
      fetchCosmosBalance({ rpcUrl, walletAddress: displayAddress, denom: tokenAddress }),
      Schedule.exponential("2 seconds", 2.0).pipe(
        Schedule.intersect(Schedule.recurs(8)),
        Schedule.whileInput(
          (error: FetchCosmosBalanceError) =>
            error._tag === "QueryBankBalanceError" &&
            error.cause instanceof Error &&
            error.cause.message.includes("HTTP")
        )
      )
    )

    yield* Effect.sync(() => {
      writeData(RawTokenBalance.make(Option.some(TokenRawAmount.make(BigInt(balance)))))
      writeError(Option.none())
    })
  }).pipe(
    Effect.tapError(error =>
      Effect.sync(() => {
        writeError(Option.some(error))
      })
    ),
    Effect.catchAll(_ => Effect.succeed(null))
  )

  return Effect.repeat(
    fetcherPipeline,
    Schedule.addDelay(Schedule.repeatForever, () => refetchInterval)
  ).pipe(Effect.scoped, Effect.provide(FetchHttpClient.layer))
}
