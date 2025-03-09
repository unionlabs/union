import { Data, Effect, Option, Schema, Schedule } from "effect"
import { fetchDecode } from "$lib/utils/queries"
import type { DurationInput } from "effect/Duration"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { Chain } from "$lib/schema/chain"
import { type AddressCosmosCanonical, AddressCosmosDisplay } from "$lib/schema/address"
import { FetchHttpClient, type HttpClientError } from "@effect/platform"
import { fromHexString, FromHexError } from "$lib/utils/hex"
import { withTracerDisabledWhen } from "@effect/platform/HttpClient"
import type { ParseError } from "effect/ParseResult"

export class NoRestRpcError extends Data.TaggedError("NoRestRpcError")<{
  chain: Chain
}> {}

export type FetchCosmosBalanceError =
  | ParseError
  | QueryBankBalanceError
  | Base64EncodeError
  | NoRestRpcError
  | FromHexError
  | Error
  | HttpClientError.HttpClientError

class QueryBankBalanceError extends Data.TaggedError("QueryBankBalanceError")<{
  cause: unknown
}> {}

export class Base64EncodeError extends Data.TaggedError("Base64EncodeError")<{
  cause: unknown
}> {}

// Schema for the balance response from Cosmos chain
export const CosmosBalanceSchema = Schema.Struct({
  balance: Schema.Struct({
    amount: TokenRawAmount
  })
})

// Schema for CW20 balance response
export const Cw20BalanceSchema = Schema.Struct({
  data: Schema.Struct({
    balance: TokenRawAmount
  })
})

const fetchCw20Balance = ({
  rpcUrl,
  contractAddress,
  walletAddress
}: {
  rpcUrl: URL
  contractAddress: AddressCosmosDisplay
  walletAddress: AddressCosmosDisplay
}) =>
  Effect.gen(function* (_) {
    const queryJson = { balance: { address: walletAddress } }

    const base64Query = yield* Effect.try({
      try: () => btoa(JSON.stringify(queryJson)),
      catch: error => new Base64EncodeError({ cause: error })
    })

    const response = yield* fetchDecode(
      Cw20BalanceSchema,
      `${rpcUrl}/cosmwasm/wasm/v1/contract/${contractAddress}/smart/${base64Query}`
    )

    return response.data.balance
  })

const fetchCosmosBalance = ({
  rpcUrl,
  walletAddress,
  denom
}: {
  rpcUrl: URL
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
    const rpcUrl = yield* Option.match(chain.getRpcUrl("rest"), {
      onNone: () => Effect.fail(new NoRestRpcError({ chain })),
      onSome: Effect.succeed
    })

    const displayAddress = yield* chain.toCosmosDisplay(walletAddress)
    const decodedDenom = yield* fromHexString(tokenAddress)

    yield* Effect.log(
      `starting balances fetcher for ${chain.universal_chain_id}:${displayAddress}:${decodedDenom}`
    )

    let balance = yield* decodedDenom.startsWith(`${chain.addr_prefix}1`)
      ? Effect.retry(
          fetchCw20Balance({
            rpcUrl,
            contractAddress: AddressCosmosDisplay.make(decodedDenom as `${string}1${string}`),
            walletAddress: displayAddress
          }),
          Schedule.exponential("2 seconds", 2.0).pipe(Schedule.intersect(Schedule.recurs(8)))
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
