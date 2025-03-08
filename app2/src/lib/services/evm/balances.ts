import { Data, Effect, Option, Schema, Schedule } from "effect"
import { erc20Abi, fromHex, type PublicClient } from "viem"
import type { TimeoutException } from "effect/Cause"
import type { DurationInput } from "effect/Duration"
import type { GetBalanceErrorType, ReadContractErrorType } from "viem"
import { getPublicClient } from "$lib/services/evm/clients"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { NoViemChainError } from "$lib/services/evm/clients"
import type { AddressEvmCanonical } from "$lib/schema/address"
import type { Chain } from "$lib/schema/chain"
import type { CreatePublicClientError } from "$lib/services/transfer"

export type FetchBalanceError =
  | NoViemChainError
  | TimeoutException
  | ReadContractError
  | FetchNativeBalanceError
  | CreatePublicClientError

export class FetchNativeBalanceError extends Data.TaggedError("FetchNativeBalanceError")<{
  cause: GetBalanceErrorType
}> {}
export class ReadContractError extends Data.TaggedError("ReadContractError")<{
  cause: ReadContractErrorType
}> {}

// Schema for the balance response
export const BalanceSchema = Schema.Struct({
  balance: Schema.String,
  token: Schema.String,
  address: Schema.String
})

const fetchNativeBalance = ({
  client,
  walletAddress
}: {
  client: PublicClient
  walletAddress: AddressEvmCanonical
}) =>
  Effect.tryPromise({
    try: () => client.getBalance({ address: walletAddress }),
    catch: err => new FetchNativeBalanceError({ cause: err as GetBalanceErrorType })
  })

const fetchTokenBalance = ({
  client,
  tokenAddress,
  walletAddress
}: {
  client: PublicClient
  tokenAddress: TokenRawDenom
  walletAddress: AddressEvmCanonical
}) =>
  Effect.tryPromise({
    try: () =>
      client.readContract({
        address: tokenAddress,
        abi: erc20Abi,
        functionName: "balanceOf",
        args: [walletAddress]
      }),
    catch: err => new ReadContractError({ cause: err as ReadContractErrorType })
  })

export const createBalanceQuery = ({
  chain,
  tokenAddress,
  walletAddress,
  refetchInterval,
  writeData,
  writeError
}: {
  chain: Chain
  tokenAddress: TokenRawDenom
  walletAddress: AddressEvmCanonical
  refetchInterval: DurationInput
  writeData: (data: RawTokenBalance) => void
  writeError: (error: Option.Option<FetchBalanceError>) => void
}) => {
  const fetcherPipeline = Effect.gen(function* (_) {
    yield* Effect.log(`starting balances fetcher for ${walletAddress}:${tokenAddress}`)
    const client = yield* getPublicClient(chain)

    const balance = yield* Effect.retry(
      fromHex(tokenAddress, "string") === "native"
        ? fetchNativeBalance({ client, walletAddress })
        : fetchTokenBalance({ client, tokenAddress, walletAddress }),
      Schedule.exponential("2 seconds", 2.0).pipe(
        Schedule.intersect(Schedule.recurs(8)),
        Schedule.whileInput(
          (error: FetchBalanceError) =>
            (error._tag === "ReadContractError" || error._tag === "FetchNativeBalanceError") &&
            error.cause?.message?.includes("HTTP request failed")
        )
      )
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
  )
}
