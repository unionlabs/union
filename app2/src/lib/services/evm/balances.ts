import { Data, Effect, Option, Schema } from "effect"
import { erc20Abi, type PublicClient } from "viem"
import type { TimeoutException } from "effect/Cause"
import type { GetBalanceErrorType, ReadContractErrorType } from "viem"
import { getPublicClient } from "$lib/services/evm/clients"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "$lib/schema/token"
import type { NoViemChainError } from "$lib/services/evm/clients"
import type { AddressEvmCanonical } from "$lib/schema/address"
import type { Chain } from "$lib/schema/chain"
import type { CreatePublicClientError } from "$lib/services/transfer"
import { fromHexString, type FromHexError } from "$lib/utils/hex"
import { evmBalanceRetrySchedule } from "$lib/constants/schedules"

export type FetchEvmBalanceError =
  | NoViemChainError
  | FromHexError
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

const fetchEvmGasBalance = ({
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

const fetchEvmErc20Balance = ({
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

// Core function to fetch a single EVM balance
export const fetchEvmBalance = ({
  chain,
  tokenAddress,
  walletAddress
}: {
  chain: Chain
  tokenAddress: TokenRawDenom
  walletAddress: AddressEvmCanonical
}) => {
  return Effect.gen(function* (_) {
    const client = yield* getPublicClient(chain)
    const decodedDenom = yield* fromHexString(tokenAddress)

    yield* Effect.log(
      `fetching balance for ${chain.universal_chain_id}:${walletAddress}:${tokenAddress}`
    )

    const fetchBalance =
      decodedDenom === "native"
        ? fetchEvmGasBalance({ client, walletAddress })
        : fetchEvmErc20Balance({ client, tokenAddress, walletAddress })

    const balance = yield* Effect.retry(fetchBalance, evmBalanceRetrySchedule)

    return RawTokenBalance.make(Option.some(TokenRawAmount.make(balance)))
  })
}
