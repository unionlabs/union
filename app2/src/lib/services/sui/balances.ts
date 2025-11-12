import { type FromHexError, fromHexString } from "$lib/utils/hex"
import { Sui } from "@unionlabs/sdk-sui"
import type { Chain } from "@unionlabs/sdk/schema"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "@unionlabs/sdk/schema"
import { Data, Effect, Option, Schema, Unify } from "effect"
import { getSuiPublicClient, NoSuiRpcError } from "./clients"

export class ReadSuiCoinError extends Data.TaggedError("ReadSuiCoinError")<{ cause: unknown }> {}

export type FetchSuiBalanceError =
  | NoSuiRpcError
  | FromHexError
  | ReadSuiCoinError
  | Sui.CreatePublicClientError

export const BalanceSchema = Schema.Struct({
  balance: Schema.String,
  token: Schema.String,
  address: Schema.String,
})

export const fetchSuiBalance = ({
  chain,
  tokenAddress,
  walletAddress,
}: {
  chain: Chain
  tokenAddress: TokenRawDenom
  walletAddress: string
}) =>
  Effect.gen(function*() {
    console.log(tokenAddress)
    const coinType = yield* fromHexString(tokenAddress)
    console.log(coinType)

    const publicClient = yield* getSuiPublicClient(chain)

    const total = yield* Sui.readTotalCoinBalance(coinType, walletAddress).pipe(
      Effect.provideService(Sui.PublicClient, publicClient),
      Effect.mapError((cause) => new ReadSuiCoinError({ cause })),
    )

    return RawTokenBalance.make(Option.some(TokenRawAmount.make(total)))
  }).pipe(
    Effect.annotateLogs({
      universal_chain_id: chain.universal_chain_id,
      walletAddress,
      tokenAddress,
    }),
  )
