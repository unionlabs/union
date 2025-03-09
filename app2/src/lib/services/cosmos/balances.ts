import { Data, Effect, Option, Schema, Schedule } from "effect"
import type { TimeoutException } from "effect/Cause"
import { fetchDecode } from "$lib/utils/queries"
import type { DurationInput } from "effect/Duration"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { Chain } from "$lib/schema/chain"
import type { AddressCosmosCanonical, AddressCosmosDisplay } from "$lib/schema/address"
import { FetchHttpClient, type HttpClientError } from "@effect/platform"
import { fromHex } from "viem"
import { withTracerDisabledWhen } from "@effect/platform/HttpClient"
import type { ParseError } from "effect/ParseResult"

export type FetchCosmosBalanceError =
  | ParseError
  | QueryBankBalanceError
  | Base64EncodeError
  | Error
  | HttpClientError.HttpClientError

class QueryBankBalanceError extends Data.TaggedError("QueryBankBalanceError")<{
  cause: unknown
}> {}

export class Base64EncodeError extends Data.TaggedError("Base64EncodeError")<{
  cause: unknown
}> {}

export class CreateClientError extends Data.TaggedError("CreateClientError")<{
  cause: unknown
}> {}

// Schema for the balance response from Cosmos chain
export const CosmosBalanceSchema = Schema.Struct({
  balance: Schema.Struct({
    amount: Schema.BigInt
  })
})

// Schema for CW20 balance response
export const Cw20BalanceSchema = Schema.Struct({
  data: Schema.Struct({
    balance: Schema.BigInt
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
  Effect.gen(function* (_) {
    yield* Effect.log(
      `fetching CW20 balance for contract ${contractAddress} and wallet ${walletAddress}`
    )

    const queryJson = { balance: { address: walletAddress } }

    const base64Query = yield* Effect.try({
      try: () => btoa(JSON.stringify(queryJson)),
      catch: error => new Base64EncodeError({ cause: error })
    })

    const response = yield* fetchDecode(
      Cw20BalanceSchema,
      `${rpcUrl}/cosmwasm/wasm/v1/contract/${contractAddress}/smart/${base64Query}`
    )

    yield* Effect.log(`received CW20 balance response for ${contractAddress}`, response)

    return response.data.balance
  }).pipe(
    Effect.tapError(error =>
      Effect.log(`error fetching CW20 balance for ${contractAddress}`, error)
    )
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
  ).pipe(Effect.map(response => response.balance.amount))

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

    // TODO: Get RPC URL from chain config
    const rpcUrl = "https://rest.testnet-9.union.build"
    const displayAddress = yield* chain.toCosmosDisplay(walletAddress)

    const decodedDenom = yield* Effect.try({
      try: () => fromHex(tokenAddress, "string"),
      catch: error => new QueryBankBalanceError({ cause: error })
    })

    yield* Effect.log(
      `starting balances fetcher for ${chain.universal_chain_id}:${displayAddress}:${decodedDenom}`
    )

    let balance = yield* decodedDenom.startsWith("union1")
      ? Effect.retry(
          fetchCw20Balance({
            rpcUrl,
            contractAddress: decodedDenom,
            walletAddress: displayAddress
          }),
          Schedule.exponential("2 seconds", 2.0).pipe(Schedule.intersect(Schedule.recurs(2)))
        )
      : // Regular bank balance query
        Effect.retry(
          fetchCosmosBalance({ rpcUrl, walletAddress: displayAddress, denom: decodedDenom }),
          Schedule.exponential("2 seconds", 2.0).pipe(Schedule.intersect(Schedule.recurs(8)))
        )
    yield* Effect.log("fetched balance", balance)

    yield* Effect.sync(() => {
      writeData(RawTokenBalance.make(Option.some(TokenRawAmount.make(balance))))
      writeError(Option.none())
    })
  }).pipe(
    Effect.tapError(error =>
      Effect.gen(function* () {
        yield* Effect.log("writing error", error)
        writeError(Option.some(error))
      })
    ),
    Effect.catchAll(_ => Effect.succeed(null))
  )

  return Effect.repeat(
    fetcherPipeline,
    Schedule.addDelay(Schedule.repeatForever, () => refetchInterval)
  ).pipe(
    Effect.scoped,
    Effect.provide(FetchHttpClient.layer),
    withTracerDisabledWhen(() => true) // important! this prevents CORS issues: https://github.com/Effect-TS/effect/issues/4568
  )
}
