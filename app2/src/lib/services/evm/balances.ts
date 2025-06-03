import { evmBalanceRetrySchedule } from "$lib/constants/schedules"
import type { NoViemChainError } from "$lib/services/evm/clients"
import { getPublicClient } from "$lib/services/evm/clients"
import type { CreatePublicClientError } from "$lib/services/transfer"
import { type FromHexError, fromHexString } from "$lib/utils/hex"
import { GAS_DENOMS } from "@unionlabs/sdk/constants/gas-denoms"
import type { AddressEvmCanonical, Chain } from "@unionlabs/sdk/schema"
import { RawTokenBalance, TokenRawAmount, type TokenRawDenom } from "@unionlabs/sdk/schema"
import { Data, Effect, Option, Schema } from "effect"
import type { TimeoutException } from "effect/Cause"
import type { GetBalanceErrorType, ReadContractErrorType } from "viem"
import { erc20Abi, type PublicClient } from "viem"

export type FetchEvmBalanceError =
  | NoViemChainError
  | FromHexError
  | TimeoutException
  | ReadContractError
  | FetchNativeBalanceError
  | CreatePublicClientError

export class FetchNativeBalanceError extends Data.TaggedError(
  "FetchNativeBalanceError",
)<{
  cause: GetBalanceErrorType
}> {}
export class ReadContractError extends Data.TaggedError("ReadContractError")<{
  cause: ReadContractErrorType
}> {}

// Schema for the balance response
export const BalanceSchema = Schema.Struct({
  balance: Schema.String,
  token: Schema.String,
  address: Schema.String,
})

const fetchEvmGasBalance = ({
  client,
  walletAddress,
}: {
  client: PublicClient
  walletAddress: AddressEvmCanonical
}) =>
  Effect.tryPromise({
    try: () => client.getBalance({ address: walletAddress }),
    catch: (err) => new FetchNativeBalanceError({ cause: err as GetBalanceErrorType }),
  })

const fetchEvmErc20Balance = ({
  client,
  tokenAddress,
  walletAddress,
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
        args: [walletAddress],
      }),
    catch: (err) => new ReadContractError({ cause: err as ReadContractErrorType }),
  })

// Core function to fetch a single Evm balance
export const fetchEvmBalance = ({
  chain,
  tokenAddress,
  walletAddress,
}: {
  chain: Chain
  tokenAddress: TokenRawDenom
  walletAddress: AddressEvmCanonical
}) =>
  Effect.gen(function*() {
    yield* Effect.logTrace("Fetching EVM balance")
    const client = yield* getPublicClient(chain)
    const decodedDenom = yield* fromHexString(tokenAddress)

    // Check if it's a native/gas token
    const isGasToken = decodedDenom === "native"
      || GAS_DENOMS[chain.universal_chain_id].address === tokenAddress

    const fetchBalance = isGasToken
      ? fetchEvmGasBalance({ client, walletAddress })
      : fetchEvmErc20Balance({ client, tokenAddress, walletAddress })

    const balance = yield* Effect.retry(fetchBalance, evmBalanceRetrySchedule)

    return RawTokenBalance.make(Option.some(TokenRawAmount.make(balance)))
  }).pipe(
    Effect.annotateLogs({
      universalChainId: chain.universal_chain_id,
      walletAddress,
      tokenAddress,
    }),
  )
