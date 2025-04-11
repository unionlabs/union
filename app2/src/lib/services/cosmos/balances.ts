import { Data, Effect, Option, Schema } from "effect"
import { fetchDecode } from "$lib/utils/queries"
import type { Chain, NoRpcError } from "@unionlabs/sdk/schema"
import {
  type AddressCosmosCanonical,
  AddressCosmosDisplay,
  RawTokenBalance,
  TokenRawAmount,
  type TokenRawDenom
} from "@unionlabs/sdk/schema"
import { FetchHttpClient, type HttpClientError } from "@effect/platform"
import { type FromHexError, fromHexString } from "$lib/utils/hex"
import { cosmosBalanceRetrySchedule } from "$lib/constants/schedules"
import { type Base64EncodeError, toBase64 } from "$lib/utils/base64"
import { withTracerDisabledWhen } from "@effect/platform/HttpClient"
import type { ParseError } from "effect/ParseResult"

export type FetchCosmosBalanceError =
  | ParseError
  | QueryBankBalanceError
  | Base64EncodeError
  | NoRpcError
  | FromHexError
  | Error
  | HttpClientError.HttpClientError

export class QueryBankBalanceError extends Data.TaggedError("QueryBankBalanceError")<{
  cause: unknown
}> {}

export const CosmosBankBalanceSchema = Schema.Struct({
  balance: Schema.Struct({
    amount: TokenRawAmount
  })
})

export const CosmosCw20BalanceSchema = Schema.Struct({
  data: Schema.Struct({
    balance: TokenRawAmount
  })
})

const fetchCosmosCw20Balance = ({
  rpcUrl,
  contractAddress,
  walletAddress
}: {
  rpcUrl: URL
  walletAddress: AddressCosmosDisplay
  contractAddress: AddressCosmosDisplay
}) =>
  Effect.gen(function* (_) {
    const queryJson = { balance: { address: walletAddress } }

    const base64Query = yield* toBase64(queryJson)

    const response = yield* fetchDecode(
      // I'm not entirely sure why this errors, but it is typesafe
      CosmosCw20BalanceSchema,
      `${rpcUrl}/cosmwasm/wasm/v1/contract/${contractAddress}/smart/${base64Query}`
    )

    return response.data.balance
  })

const fetchCosmosBankBalance = ({
  rpcUrl,
  walletAddress,
  denom
}: {
  rpcUrl: URL
  walletAddress: AddressCosmosDisplay
  denom: string
}) =>
  fetchDecode(
    // I'm not entirely sure why this errors, but it is typesafe
    CosmosBankBalanceSchema,
    `${rpcUrl}/cosmos/bank/v1beta1/balances/${walletAddress}/by_denom?denom=${denom}`
  ).pipe(Effect.map(response => response.balance.amount))

// Core function to fetch a single Cosmos balance
export const fetchCosmosBalance = ({
  chain,
  tokenAddress,
  walletAddress
}: {
  chain: Chain
  tokenAddress: TokenRawDenom
  walletAddress: AddressCosmosCanonical
}) => {
  return Effect.gen(function* () {
    const rpcUrl = yield* chain.requireRpcUrl("rest")
    const displayAddress = yield* chain.toCosmosDisplay(walletAddress)
    const decodedDenom = yield* fromHexString(tokenAddress)

    yield* Effect.log(
      `fetching balance for ${chain.universal_chain_id}:${displayAddress}:${decodedDenom}`
    )

    const fetchBalance = decodedDenom.startsWith(`${chain.addr_prefix}1`)
      ? fetchCosmosCw20Balance({
          rpcUrl,
          walletAddress: displayAddress,
          contractAddress: AddressCosmosDisplay.make(decodedDenom as `${string}1${string}`)
        })
      : fetchCosmosBankBalance({
          rpcUrl,
          walletAddress: displayAddress,
          denom: decodedDenom
        })

    let balance = yield* Effect.retry(fetchBalance, cosmosBalanceRetrySchedule)

    return RawTokenBalance.make(Option.some(TokenRawAmount.make(balance)))
  }).pipe(
    Effect.scoped,
    Effect.provide(FetchHttpClient.layer),
    withTracerDisabledWhen(() => true) // important! this prevents CORS issues: https://github.com/Effect-TS/effect/issues/4568
  )
}
