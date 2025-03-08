import { Data, Effect, Option, Schema, Schedule } from "effect"
import type { TimeoutException } from "effect/Cause"
import { fetchDecode } from "$lib/utils/queries"
import type { DurationInput } from "effect/Duration"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { Chain } from "$lib/schema/chain"
import type { AddressCosmosCanonical, AddressCosmosDisplay } from "$lib/schema/address"
import { FetchHttpClient } from "@effect/platform"
import { fromHex } from "viem"

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

// Schema for CW20 balance response
export const Cw20BalanceSchema = Schema.Struct({
  data: Schema.Struct({
    balance: Schema.String
  })
})

const fetchCw20Balance = ({
  rpcUrl,
  contractAddress,
  walletAddress
}: {
  rpcUrl: string
  contractAddress: string
  walletAddress: AddressCosmosDisplay
}) =>
  fetchDecode(
    Cw20BalanceSchema,
    `${rpcUrl}/cosmwasm/wasm/v1/contract/${contractAddress}/smart/${Buffer.from(
      JSON.stringify({ balance: { address: walletAddress } })
    ).toString("base64")}`
  ).pipe(
    Effect.map(response => response.data.balance),
    Effect.mapError(err => new QueryBankBalanceError({ cause: err }))
  )

const fetchCosmosBalance = ({
  rpcUrl,
  walletAddress,
  denom
}: {
  rpcUrl: string
  walletAddress: AddressCosmosDisplay
  denom: string
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
  const fetcherPipeline = Effect.gen(function* () {
    if (chain.universal_chain_id !== "union.union-testnet-9")
      yield* Effect.fail(new Error("Only union supported"))
    yield* Effect.log(`starting cosmos balances fetcher for ${walletAddress}:${tokenAddress}`)

    // TODO: Get RPC URL from chain config
    const rpcUrl = "https://rest.testnet-9.union.build"
    const displayAddress = yield* chain.toCosmosDisplay(walletAddress)

    const decodedDenom = yield* Effect.try({
      try: () => fromHex(tokenAddress, "string"),
      catch: error => new QueryBankBalanceError({ cause: error })
    })

    let balance = yield* decodedDenom.startsWith("union1")
      ? Effect.retry(
          fetchCw20Balance({
            rpcUrl,
            contractAddress: decodedDenom,
            walletAddress: displayAddress
          }),
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
      : // Regular bank balance query
        Effect.retry(
          fetchCosmosBalance({ rpcUrl, walletAddress: displayAddress, denom: decodedDenom }),
          Schedule.exponential("2 seconds", 2.0).pipe(Schedule.intersect(Schedule.recurs(8)))
        )

    yield* Effect.sync(() => {
      writeData(RawTokenBalance.make(Option.some(TokenRawAmount.make(balance))))
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
